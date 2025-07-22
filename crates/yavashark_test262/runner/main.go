package main

import (
	"log"
	"path/filepath"
	"yavashark_test262_runner/ci"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
)

const (
	DEFAULT_TEST_ROOT = "test262/test"
	DEFAULT_WORKERS   = 256
)

func main() {
	config := LoadConfig()

	testRoot := filepath.Join(config.TestRootDir, config.TestDir)

	testResults := run.TestsInDir(testRoot, config.Workers)

	if config.Diff && !config.CI {
		printDiff(testResults, config.DiffFilter)
	}

	if config.CI {
		ci.RunCi(testResults, config.RepoPath, config.HistoryOnly, config.Diff, testRoot)
	} else {
		testResults.PrintResults()

		print("\n\n\n")
		_ = testResults.ComparePrev()
	}

	if config.TestDir == "" {
		testResults.Write()
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
