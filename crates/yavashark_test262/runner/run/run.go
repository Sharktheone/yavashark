package run

import (
	"log"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
	"yavashark_test262_runner/worker"
)

var SKIP = []string{
	"intl402",
	"staging",
}

func TestsInDir(testRoot string, workers int) *results.TestResults {
	jobs := make(chan string, workers*8)

	resultsChan := make(chan results.Result, workers*8)

	wg := &sync.WaitGroup{}

	wg.Add(workers)

	for i := range workers {
		go worker.Worker(i, jobs, resultsChan, wg)
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

		for _, skip := range SKIP {
			if strings.HasPrefix(filepath.Join(testRoot, path), skip) {
				log.Printf("Skipping %s", path)

				resultsChan <- results.Result{
					Status: status.SKIP,
					Msg:    "skip",
					Path:   path,
				}

				return nil
			}
		}

		jobs <- path

		return nil
	})

	close(jobs)

	wg.Wait()
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
