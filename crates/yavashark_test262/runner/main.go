package main

import (
	"log"
	"path/filepath"
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
