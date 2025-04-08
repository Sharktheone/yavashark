package main

import (
	"flag"
	"log"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
	"yavashark_test262_runner/ci"
	"yavashark_test262_runner/results"
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

	jobs := make(chan string, workers*8)

	resultsChan := make(chan results.Result, workers*8)

	wg := &sync.WaitGroup{}

	wg.Add(workers)

	for i := range workers {
		go worker(i, jobs, resultsChan, wg)
	}

	num := countTests(testRoot)

	testResults := results.New(num)

	go func() {
		for res := range resultsChan {
			testResults.Add(res)
		}
	}()

	now := time.Now()
	_ = filepath.Walk(testRoot, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			//log.Printf("Failed to get file info for %s: %v", path, err)
			return nil
		}

		if info.IsDir() {
			return nil
		}

		if strings.Contains(path, "_FIXTURE") {
			return nil
		}

		jobs <- path

		return nil
	})

	close(jobs)

	wg.Wait()
	log.Printf("Finished running %d tests in %s", num, time.Since(now).String())

	close(resultsChan)

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

func countTests(path string) uint32 {
	var num uint32 = 0

	_ = filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}

		if info == nil {
			log.Printf("Failed to get file info for %s", path)
			return nil
		}

		if info.IsDir() {
			return nil
		}

		if strings.Contains(path, "_FIXTURE") {
			return nil
		}

		num++

		return nil
	})

	return num
}
