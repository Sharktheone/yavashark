package main

import (
	"log"
	"path/filepath"
	"time"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/ci"
	"yavashark_test262_runner/progress"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
	"yavashark_test262_runner/timing"
)

const (
	DEFAULT_TEST_ROOT = "test262/test"
)

func main() {
	config := LoadConfig()
	type preparedResult struct {
		data *run.Prepared
		err  error
	}
	var loading chan preparedResult
	if config.Rebuild && !config.CI {
		loading = make(chan preparedResult, 1)
		go func() {
			p, err := run.Prepare([]string{config.testPath()}, config.runConfig())
			loading <- preparedResult{p, err}
		}()
	}

	if config.Rebuild {
		buildConfig := build.Config{
			Rebuild:  true,
			Mode:     config.BuildMode,
			Compiler: config.BuildCompiler,
		}
		if err := build.RebuildEngine(buildConfig); err != nil {
			log.Fatalf("Failed to rebuild engine: %v", err)
		}
	}

	if loading != nil {
		p := <-loading
		if p.err != nil {
			log.Fatal(p.err)
		}
		config.prepared = p.data
	}
	if config.Rebuild && config.Engine == "" && config.BuildMode == build.BuildModeDebug {
		config.Engine = "../../target/debug/yavashark_test262"
	}

	if config.Bench {
		root := filepath.Join(config.TestRootDir, config.TestDir)
		if config.FilterPath != "" {
			root = filepath.Join(config.TestRootDir, NormalizeFilterPath(config.FilterPath, config.TestRootDir))
		}
		timeout, _ := time.ParseDuration(config.ProbeTimeout)
		if err := run.Benchmark([]string{root}, run.BenchConfig{RunConfig: config.runConfig(), Samples: config.BenchSamples, Repeats: config.BenchRepeats, ProbeTimeout: timeout, Output: config.BenchOutput}); err != nil {
			log.Fatal(err)
		}
		return
	}

	if config.FilterPath != "" {
		runFilteredTests(config)
		return
	}

	testRoot := filepath.Join(config.TestRootDir, config.TestDir)

	runConfig := config.runConfig()

	testResults, summary := run.TestsInDir(testRoot, runConfig)
	if testResults.RunnerError > 0 {
		log.Fatal("runner errors; refusing to replace conformance results")
	}

	if config.Diff && !config.CI && config.Selection != "quick" {
		printDiff(testResults, config.DiffFilter)
	}

	if !config.CI {
		progress.PrintSummary(summary)
	}

	if config.CI {
		ci.RunCi(testResults, config.RepoPath, config.HistoryOnly, config.Diff, testRoot)
	} else if config.Verbose {
		testResults.PrintResults(config.ShowStats)

		print("\n\n\n")
		_ = testResults.ComparePrev()
	}

	if config.Selection == "quick" || config.Output != "" {
		path := config.Output
		if path == "" {
			path = "results-quick.json"
		}
		if err := calibration.Save(path, testResults.TestResults); err != nil {
			log.Fatal(err)
		}
	} else if config.TestDir == "" {
		testResults.Write()
	}

	if config.Timings {
		timing.PrintTimings()
	}
}

func runFilteredTests(config *Config) {
	normalizedFilter := NormalizeFilterPath(config.FilterPath, config.TestRootDir)

	testRoot := filepath.Join(config.TestRootDir, normalizedFilter)

	log.Printf("Running filtered tests in: %s", testRoot)

	runConfig := config.runConfig()

	filteredResults, filteredSummary := run.TestsInDir(testRoot, runConfig)
	if filteredResults.RunnerError > 0 {
		log.Fatal("runner errors; refusing to replace conformance results")
	}

	if config.Selection == "quick" || config.Output != "" {
		progress.PrintSummary(filteredSummary)
		path := config.Output
		if path == "" {
			path = "results-quick.json"
		}
		if err := calibration.Save(path, filteredResults.TestResults); err != nil {
			log.Fatal(err)
		}
		return
	}
	prevResults, err := results.LoadResults()
	if err != nil {
		log.Printf("Warning: Could not load previous results: %v", err)
		prevResults = nil
	}

	var mergedResults *results.TestResults
	if prevResults != nil {
		mergedResults = filteredResults.MergeInto(prevResults)
	} else {
		mergedResults = filteredResults
	}

	mergedSummary := progress.Summary{
		Passed:            mergedResults.Passed,
		Failed:            mergedResults.Failed,
		Skipped:           mergedResults.Skipped,
		Timeout:           mergedResults.Timeout,
		Crashed:           mergedResults.Crashed,
		ParseError:        mergedResults.ParseError,
		ParseSuccessError: mergedResults.ParseSuccessError,
		NotImplemented:    mergedResults.NotImplemented,
		RunnerError:       mergedResults.RunnerError,
		Total:             mergedResults.Total,
	}

	if prevResults != nil {
		prevTestResults := results.FromResults(prevResults)
		mergedSummary.PassGained = int32(mergedResults.Passed) - int32(prevTestResults.Passed)
		mergedSummary.FailGained = int32(mergedResults.Failed) - int32(prevTestResults.Failed)
	}

	filteredSimple := progress.SimpleSummary{
		Passed:            filteredSummary.Passed,
		Failed:            filteredSummary.Failed,
		Skipped:           filteredSummary.Skipped,
		Timeout:           filteredSummary.Timeout,
		Crashed:           filteredSummary.Crashed,
		ParseError:        filteredSummary.ParseError,
		ParseSuccessError: filteredSummary.ParseSuccessError,
		NotImplemented:    filteredSummary.NotImplemented,
		RunnerError:       filteredSummary.RunnerError,
		Total:             filteredSummary.Total,
	}

	if config.Diff {
		printDiff(filteredResults, config.DiffFilter)
	}

	progress.PrintSummaryWithFilter(mergedSummary, filteredSimple, normalizedFilter)

	mergedResults.Write()

	if config.Verbose {
		filteredResults.PrintResults(config.ShowStats)
	}

	if config.Timings {
		timing.PrintTimings()
	}
}

func printDiff(testResults *results.TestResults, diffFilter string) {
	diff, err := testResults.ComputeDiffPrev()
	if err != nil {
		log.Printf("Failed to compute diff: %v", err)
		return
	}

	if diffFilter == "" {
		diff.PrintGrouped()
	} else {
		filter, err := results.ParseFilter(diffFilter)
		if err != nil {
			log.Printf("Failed to parse diff filter: %v", err)
			return
		}

		diff.PrintGroupedFilter(filter)
	}
}

func (config *Config) runConfig() run.RunConfig {
	return run.RunConfig{Prepared: config.prepared, Workers: config.Workers, RiskyWorkers: config.RiskyWorkers, RiskyCPUs: config.RiskyCPUs, CPUCount: config.CPUCount, Selection: config.Selection, Coverage: config.Coverage, CalibrationPath: config.CalibrationPath, CalibrationKey: config.CalibrationKey, HistoryPath: config.HistoryPath, Engine: config.Engine, ReportPath: config.ReportPath, Skips: config.Skips, Timings: config.Timings, Timeout: config.Timeout, Interactive: config.Interactive}
}

func (config *Config) testPath() string {
	if config.FilterPath != "" {
		return filepath.Join(config.TestRootDir, NormalizeFilterPath(config.FilterPath, config.TestRootDir))
	}
	return filepath.Join(config.TestRootDir, config.TestDir)
}
