package main

import (
	"sync"
	"yavashark_test262_runner/results"
)

func worker(id int, jobs <-chan string, results chan<- results.Result, wg *sync.WaitGroup) {
	defer wg.Done()

	for path := range jobs {
		res := runTest(path)

		results <- res
	}
}
