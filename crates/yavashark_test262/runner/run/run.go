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
	"yavashark_test262_runner/test"
	"yavashark_test262_runner/worker"
)

var SKIP = []string{
	"intl402",
	"staging",
}

type RunConfig struct {
	Workers     int
	Skips       bool
	Timings     bool
	Timeout     time.Duration
	Interactive bool
}

func TestsInDir(testRoot string, config RunConfig) (*results.TestResults, progress.Summary) {
	// Set the timeout for tests
	test.SetTimeout(config.Timeout)

	jobs := make(chan string, config.Workers*8)

	resultsChan := make(chan results.Result, config.Workers*8)

	wg := &sync.WaitGroup{}

	wg.Add(config.Workers)

	num := countTests(testRoot)

	// Load previous results for delta calculation
	prevResults, _ := results.LoadResults()
	var prevResultsMap map[string]status.Status
	if prevResults != nil {
		prevResultsMap = make(map[string]status.Status)
		for _, r := range prevResults {
			prevResultsMap[r.Path] = r.Status
		}
	}

	progressTracker := progress.NewProgressTracker(num, config.Interactive, prevResultsMap)

	for i := range config.Workers {
		go worker.Worker(i, jobs, resultsChan, wg, config.Timings)
	}

	testResults := results.New(num)

	// WaitGroup for result processing
	resultsDone := make(chan struct{})

	// Goroutine to process results and update progress
	go func() {
		for res := range resultsChan {
			testResults.Add(res)
			progressTracker.Add(res.Status, res.Path)
		}
		close(resultsDone)
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
		if config.Skips {
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

	// Get statistics only for tests we're actually running
	min, max, avg, fastCount, mediumCount, slowCount, riskCount := scheduler.GetStatisticsForTests(timingData, testPaths)
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
	close(resultsChan)

	<-resultsDone

	summary := progressTracker.Finish()

	fmt.Printf("\n")

	log.Printf("Finished running %d tests in %s", num, time.Since(now).String())

	return testResults, summary
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
