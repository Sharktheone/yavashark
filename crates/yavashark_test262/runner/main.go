package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"yavashark_test262_runner/status"
)

const (
	TEST_ROOT  = "test262/test"
	SKIP_EXTRA = true

	WORKERS = 128
)

func main() {
	jobs := make(chan string, WORKERS*8)

	results := make(chan Result, WORKERS*8)

	wg := &sync.WaitGroup{}

	wg.Add(WORKERS)

	for i := range WORKERS {
		go worker(i, jobs, results, wg)
	}

	num := countTests(TEST_ROOT)

	testResults := make([]Result, 0, num)

	go func() {
		for res := range results {
			testResults = append(testResults, res)
		}
	}()

	filepath.Walk(TEST_ROOT, func(path string, info os.FileInfo, err error) error {
		if info.IsDir() {
			return nil
		}

		jobs <- path

		return nil
	})

	close(jobs)

	wg.Wait()

	close(results)

	passed := 0
	failed := 0
	skipped := 0
	notImplemented := 0
	runnerError := 0
	crashed := 0
	timeout := 0
	parse := 0

	for _, res := range testResults {
		switch res.Status {
		case status.PASS:
			passed++
		case status.FAIL:
			failed++
		case status.SKIP:
			skipped++
		case status.NOT_IMPLEMENTED:
			notImplemented++
		case status.RUNNER_ERROR:
			runnerError++
		case status.CRASH:
			crashed++
		case status.PARSE_ERROR:
			parse++
		case status.TIMEOUT:
			timeout++
		}
	}

	fmt.Printf("Passed: %d, %f%%\n", passed, float64(passed)/float64(num)*100)
	fmt.Printf("Failed: %d, %f%%\n", failed, float64(failed)/float64(num)*100)
	fmt.Printf("Skipped: %d, %f%%\n", skipped, float64(skipped)/float64(num)*100)
	fmt.Printf("Not Implemented: %d, %f%%\n", notImplemented, float64(notImplemented)/float64(num)*100)
	fmt.Printf("Runner Error: %d, %f%%\n", runnerError, float64(runnerError)/float64(num)*100)
	fmt.Printf("Crashed: %d, %f%%\n", crashed, float64(crashed)/float64(num)*100)
	fmt.Printf("Timeout: %d, %f%%\n", timeout, float64(timeout)/float64(num)*100)
	fmt.Printf("Parse Error: %d, %f%%\n", parse, float64(parse)/float64(num)*100)

	err := writeResults(testResults)
	if err != nil {
		panic(err)
	}
}

func countTests(path string) int {
	num := 0

	filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
		if info.IsDir() {
			return nil
		}

		num++

		return nil
	})

	return num
}
