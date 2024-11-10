package main

import "sync"

func worker(id int, jobs <-chan string, results chan<- Result, wg *sync.WaitGroup) {
	defer wg.Done()

	for path := range jobs {
		res := runTest(path)

		results <- res
	}
}
