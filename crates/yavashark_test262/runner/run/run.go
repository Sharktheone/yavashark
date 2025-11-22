package run

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
	"yavashark_test262_runner/progress"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/scheduler"
	"yavashark_test262_runner/status"
	"yavashark_test262_runner/worker"
)

var SKIP = []string{
	"intl402",
	"staging",
}

func TestsInDir(testRoot string, workers int, skips bool, timings bool) *results.TestResults {
	jobs := make(chan string, workers*8)

	resultsChan := make(chan results.Result, workers*8)

	wg := &sync.WaitGroup{}

	wg.Add(workers)

	num := countTests(testRoot)
	progressTracker := progress.NewProgressTracker(num)

	for i := range workers {
		go worker.Worker(i, jobs, resultsChan, wg, timings)
	}

	testResults := results.New(num)

	// Goroutine to process results and update progress
	go func() {
		for res := range resultsChan {
			testResults.Add(res)
			progressTracker.Add(res.Status)
		}
	}()

	var testPaths []string
	var skippedPaths []string

	_ = filepath.Walk(testRoot, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}

		if info.IsDir() {
			return nil
		}

		if strings.Contains(path, "_FIXTURE") {
			return nil
		}

		p, err := filepath.Rel(testRoot, path)
		if err != nil {
			log.Printf("Failed to get relative path for %s: %v", path, err)
			return nil
		}

		shouldSkip := false
		if skips {
			for _, skip := range SKIP {
				if strings.HasPrefix(p, skip) {
					skippedPaths = append(skippedPaths, path)
					shouldSkip = true
					break
				}
			}
		}

		if !shouldSkip {
			testPaths = append(testPaths, path)
		}

		return nil
	})

	timingData := scheduler.LoadTestTimings("results.json")

	scheduler.EnrichTimingsWithFallback(timingData, testPaths)

	min, max, avg, fastCount, mediumCount, slowCount, riskCount := scheduler.GetStatistics(timingData)
	log.Printf("Timing statistics - Min: %v, Max: %v, Avg: %v", min, max, avg)
	log.Printf("Test distribution - Fast: %d, Medium: %d, Slow: %d, Risky: %d",
		fastCount, mediumCount, slowCount, riskCount)

	scheduledJobs := scheduler.ScheduleTests(testPaths, timingData)

	now := time.Now()

	go func() {
		for _, job := range scheduledJobs {
			jobs <- job.Path
		}

		for _, path := range skippedPaths {
			resultsChan <- results.Result{
				Status:   status.SKIP,
				Msg:      "skip",
				Path:     path,
				MemoryKB: 0,
				Duration: 0,
			}
		}

		close(jobs)
	}()

	wg.Wait()

	fmt.Printf("\n")

	log.Printf("Finished running %d tests in %s", num, time.Since(now).String())

	close(resultsChan)

	return testResults
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
