package worker

import (
	"sync"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/test"
)

func Worker(id int, jobs <-chan string, results chan<- results.Result, wg *sync.WaitGroup) {
	defer wg.Done()

	for path := range jobs {
		res := test.RunTest(path)

		results <- res
	}
}
