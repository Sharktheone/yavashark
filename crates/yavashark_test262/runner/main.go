package main

import (
	"flag"
	"log"
	"path/filepath"
	"yavashark_test262_runner/ci"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
)

const (
	DEFAULT_TEST_ROOT = "test262/test"

	DEFAULT_WORKERS = 128
)

func main() {
	ciEnabled := flag.Bool("ci", false, "Enable CI mode to commit results")
	repoPath := flag.String("repo", "", "Path to external repository for CI results")
	historyOnly := flag.Bool("history-only", false, "Only generate the history file (skip git commit)")
	workers := *flag.Int("workers", DEFAULT_WORKERS, "Number of workers")
	testRootDir := flag.String("test_root", DEFAULT_TEST_ROOT, "Path to test root directory")
	diff := flag.Bool("diff", true, "Diff to use for CI results")
	diffFilter := flag.String("dfilter", "", "Diff filter to use for CI results")
	testdir := flag.String("testdir", "", "Path in the test directory")

	flag.Parse()

	testRoot := filepath.Join(*testRootDir, *testdir)

	testResults := run.TestsInDir(testRoot, workers)

	if *diff && !*ciEnabled {
		printDiff(testResults, *diffFilter)
	}

	testResults.PrintResults()

	//if *testdir == "" {
	//	testResults.Write()
	//}

	print("\n\n\n")

	if *ciEnabled {
		ci.RunCi(testResults, *repoPath, *historyOnly, *diff, testRoot)
	} else {
		_ = testResults.ComparePrev()

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
