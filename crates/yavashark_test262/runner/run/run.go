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
	FailedOnly  bool // Only run tests that are currently failing
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

// TestSpecificPaths runs a specific list of test paths (files or directories)
// Returns the test results and a list of paths that were skipped due to errors
func TestSpecificPaths(paths []string, config RunConfig) (*results.TestResults, progress.Summary, []string) {
	// Set the timeout for tests
	test.SetTimeout(config.Timeout)

	// Load previous results for failedOnly filtering and delta calculation
	prevResults, _ := results.LoadResults()
	var prevResultsMap map[string]status.Status
	if prevResults != nil {
		prevResultsMap = make(map[string]status.Status)
		for _, r := range prevResults {
			prevResultsMap[r.Path] = r.Status
		}
	}

	// Expand paths - if a path is a directory, walk it to find all tests
	var testPaths []string
	var skippedPaths []string
	var errorPaths []string

	for _, p := range paths {
		info, err := os.Stat(p)
		if err != nil {
			errorPaths = append(errorPaths, p)
			continue
		}

		if info.IsDir() {
			// Walk the directory to find all test files
			_ = filepath.Walk(p, func(path string, info os.FileInfo, err error) error {
				if err != nil {
					return nil
				}
				if info.IsDir() {
					return nil
				}
				if strings.Contains(path, "_FIXTURE") {
					return nil
				}

				// Check for skip directories if enabled
				if config.Skips {
					for _, skip := range SKIP {
						if strings.Contains(path, "/"+skip+"/") || strings.HasPrefix(path, skip) {
							skippedPaths = append(skippedPaths, path)
							return nil
						}
					}
				}

				// If failedOnly, skip tests that are currently passing
				if config.FailedOnly && prevResultsMap != nil {
					if prevStatus, ok := prevResultsMap[path]; ok && prevStatus == status.PASS {
						return nil // skip passing tests
					}
				}

				testPaths = append(testPaths, path)
				return nil
			})
		} else {
			// Single file
			if strings.Contains(p, "_FIXTURE") {
				continue
			}
			// If failedOnly, skip tests that are currently passing
			if config.FailedOnly && prevResultsMap != nil {
				if prevStatus, ok := prevResultsMap[p]; ok && prevStatus == status.PASS {
					continue // skip passing tests
				}
			}
			testPaths = append(testPaths, p)
		}
	}

	if len(testPaths) == 0 && len(skippedPaths) == 0 {
		// No tests to run
		return results.New(0), progress.Summary{}, errorPaths
	}

	jobs := make(chan string, config.Workers*8)
	resultsChan := make(chan results.Result, config.Workers*8)
	wg := &sync.WaitGroup{}
	wg.Add(config.Workers)

	num := uint32(len(testPaths) + len(skippedPaths))

	progressTracker := progress.NewProgressTracker(num, config.Interactive, prevResultsMap)

	for i := range config.Workers {
		go worker.Worker(i, jobs, resultsChan, wg, config.Timings)
	}

	testResults := results.New(num)
	resultsDone := make(chan struct{})

	go func() {
		for res := range resultsChan {
			testResults.Add(res)
			progressTracker.Add(res.Status, res.Path)
		}
		close(resultsDone)
	}()

	// Load timing data for scheduling
	timingData := scheduler.LoadTestTimings("results.json")
	scheduler.EnrichTimingsWithFallback(timingData, testPaths)
	scheduledJobs := scheduler.ScheduleTests(testPaths, timingData)

	now := time.Now()

	go func() {
		for _, job := range scheduledJobs {
			jobs <- job.Path
		}

		// Add skipped paths as SKIP results
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

	log.Printf("Finished running %d tests in %s", len(testPaths), time.Since(now).String())

	return testResults, summary, errorPaths
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
