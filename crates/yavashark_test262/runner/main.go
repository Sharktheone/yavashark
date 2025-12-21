package main

import (
	"log"
	"path/filepath"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/ci"
	"yavashark_test262_runner/progress"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
	"yavashark_test262_runner/timing"
)

const (
	DEFAULT_TEST_ROOT = "test262/test"
	DEFAULT_WORKERS   = 1024
)

func main() {
	config := LoadConfig()

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

	if config.FilterPath != "" {
		runFilteredTests(config)
		return
	}

	testRoot := filepath.Join(config.TestRootDir, config.TestDir)

	runConfig := run.RunConfig{
		Workers:     config.Workers,
		Skips:       config.Skips,
		Timings:     config.Timings,
		Timeout:     config.Timeout,
		Interactive: config.Interactive,
	}

	testResults, summary := run.TestsInDir(testRoot, runConfig)

	if config.Diff && !config.CI {
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

	if config.TestDir == "" {
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

	runConfig := run.RunConfig{
		Workers:     config.Workers,
		Skips:       config.Skips,
		Timings:     config.Timings,
		Timeout:     config.Timeout,
		Interactive: config.Interactive,
	}

	filteredResults, filteredSummary := run.TestsInDir(testRoot, runConfig)

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
