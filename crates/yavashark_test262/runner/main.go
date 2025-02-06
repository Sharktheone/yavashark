package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"
	"yavashark_test262_runner/status"
)

const (
	DEFAULT_TEST_ROOT = "test262/test"

	DEFAULT_WORKERS = 128
)

// New types for the CI report
type Summary struct {
	Passed         int    `json:"passed"`
	Failed         int    `json:"failed"`
	Skipped        int    `json:"skipped"`
	NotImplemented int    `json:"not_implemented"`
	RunnerError    int    `json:"runner_error"`
	Crashed        int    `json:"crashed"`
	Timeout        int    `json:"timeout"`
	ParseError     int    `json:"parse_error"`
	Total          int    `json:"total"`
	Timestamp      string `json:"timestamp"`
	CommitHash     string `json:"commit_hash"`
}

type History struct {
	Runs []Summary `json:"runs"`
}

type DirectorySummary struct {
	Directory      string `json:"directory"`
	Passed         int    `json:"passed"`
	Failed         int    `json:"failed"`
	Skipped        int    `json:"skipped"`
	NotImplemented int    `json:"not_implemented"`
	RunnerError    int    `json:"runner_error"`
	Crashed        int    `json:"crashed"`
	Timeout        int    `json:"timeout"`
	ParseError     int    `json:"parse_error"`
	Total          int    `json:"total"`
}

func main() {
	ciEnabled := flag.Bool("ci", false, "Enable CI mode to commit results")
	repoPath := flag.String("repo", "", "Path to external repository for CI results")
	historyOnly := flag.Bool("history-only", false, "Only generate the history file (skip git commit)")
	workers := *flag.Int("workers", DEFAULT_WORKERS, "Number of workers")
	testRoot := flag.String("test_root", DEFAULT_TEST_ROOT, "Path to test root directory")
	flag.Parse()

	jobs := make(chan string, workers*8)

	results := make(chan Result, workers*8)

	wg := &sync.WaitGroup{}

	wg.Add(workers)

	for i := range workers {
		go worker(i, jobs, results, wg)
	}

	num := countTests(*testRoot)

	testResults := make([]Result, 0, num)

	go func() {
		for res := range results {
			testResults = append(testResults, res)
		}
	}()

	filepath.Walk(*testRoot, func(path string, info os.FileInfo, err error) error {
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

	if *ciEnabled {
		if *repoPath == "" {
			fmt.Println("CI mode enabled but no repository path specified via --repo")
			os.Exit(1)
		}

		overallSummary := Summary{
			Passed:         passed,
			Failed:         failed,
			Skipped:        skipped,
			NotImplemented: notImplemented,
			RunnerError:    runnerError,
			Crashed:        crashed,
			Timeout:        timeout,
			ParseError:     parse,
			Total:          num,
			Timestamp:      time.Now().Format(time.RFC3339),
		}
		commitBytes, err := exec.Command("git", "rev-parse", "HEAD").Output()
		if err != nil {
			log.Printf("Failed to get commit hash: %v", err)
			overallSummary.CommitHash = "unknown"
		} else {
			overallSummary.CommitHash = strings.TrimSpace(string(commitBytes))
		}

		commitTimeBytes, err := exec.Command("git", "show", "-s", "--format=%ct", "HEAD").Output()
		if err != nil {
			log.Printf("Failed to get commit time: %v", err)
		} else {
			commitEpochStr := strings.TrimSpace(string(commitTimeBytes))
			commitEpoch, err2 := strconv.ParseInt(commitEpochStr, 10, 64)
			if err2 != nil {
				log.Printf("Failed to parse commit timestamp: %v", err2)
			} else {
				overallSummary.Timestamp = time.Unix(commitEpoch, 0).Format(time.RFC3339)
			}
		}

		if err := runCI(testResults, overallSummary, *repoPath, *historyOnly, *testRoot); err != nil {
			panic(err)
		}
	}
}

func countTests(path string) int {
	num := 0

	filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
		if info == nil {
			log.Printf("Failed to get file info for %s", path)
			return nil
		}

		if info.IsDir() {
			return nil
		}

		num++

		return nil
	})

	return num
}

func runCI(testResults []Result, overall Summary, repo string, historyOnly bool, root string) error {
	if err := generateHistoryFile(repo, overall); err != nil {
		return err
	}

	if historyOnly {
		return nil
	}

	resultsDir := filepath.Join(repo, "results")
	if err := os.MkdirAll(resultsDir, 0755); err != nil {
		return err
	}

	dirSummaries := map[string]*DirectorySummary{}

	for _, res := range testResults {
		relPath, err := filepath.Rel(root, res.Path)
		if err != nil {
			relPath = res.Path
		}

		outFile := filepath.Join(resultsDir, relPath) + ".json"
		os.MkdirAll(filepath.Dir(outFile), 0755)
		data, err := json.Marshal(res)
		if err != nil {
			return err
		}
		if err := os.WriteFile(outFile, data, 0644); err != nil {
			return err
		}

		relativeDir := filepath.Dir(relPath)
		if relativeDir == "." {
			relativeDir = ""
		}

		if relativeDir != "" {
			parts := strings.Split(relativeDir, string(filepath.Separator))
			accum := ""
			for _, part := range parts {
				if accum == "" {
					accum = part
				} else {
					accum = filepath.Join(accum, part)
				}
				if _, exists := dirSummaries[accum]; !exists {
					dirSummaries[accum] = &DirectorySummary{Directory: accum}
				}
			}
		}
		if _, exists := dirSummaries[""]; !exists {
			dirSummaries[""] = &DirectorySummary{Directory: ""}
		}

		ds, exists := dirSummaries[relativeDir]
		if !exists {
			ds = &DirectorySummary{Directory: relativeDir}
			dirSummaries[relativeDir] = ds
		}
		ds.Total++
		switch res.Status {
		case status.PASS:
			ds.Passed++
		case status.FAIL:
			ds.Failed++
		case status.SKIP:
			ds.Skipped++
		case status.NOT_IMPLEMENTED:
			ds.NotImplemented++
		case status.RUNNER_ERROR:
			ds.RunnerError++
		case status.CRASH:
			ds.Crashed++
		case status.PARSE_ERROR:
			ds.ParseError++
		case status.TIMEOUT:
			ds.Timeout++
		}
	}

	for dir := range dirSummaries {
		agg := computeAggregate(dir, dirSummaries)
		var sumPath string
		if dir == "" {
			sumPath = filepath.Join(resultsDir, "summary.json")
		} else {
			sumPath = filepath.Join(resultsDir, dir, "summary.json")
		}
		data, err := json.Marshal(agg)
		if err != nil {
			return err
		}
		if err := os.WriteFile(sumPath, data, 0644); err != nil {
			return err
		}
	}

	cmd := exec.Command("git", "add", ".")
	cmd.Dir = repo
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("git add failed: %v, output: %s", err, out)
	}
	commitMsg := fmt.Sprintf("CI: Updated test results at %s", time.Now().Format(time.RFC3339))
	cmd = exec.Command("git", "commit", "-m", commitMsg)
	cmd.Dir = repo
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("git commit failed: %v, output: %s", err, out)
	}

	return nil
}

func generateHistoryFile(repo string, overall Summary) error {
	historyFile := filepath.Join(repo, "history.json")
	var history History
	if b, err := os.ReadFile(historyFile); err == nil {
		if err = json.Unmarshal(b, &history); err != nil {
			log.Panicf("Failed to unmarshal history file: %v", err)
		}
	}
	history.Runs = append(history.Runs, overall)
	historyData, err := json.Marshal(history)
	if err != nil {
		return err
	}
	if err := os.WriteFile(historyFile, historyData, 0644); err != nil {
		return err
	}
	return nil
}

func computeAggregate(dir string, summaries map[string]*DirectorySummary) DirectorySummary {
	base, exists := summaries[dir]
	var agg DirectorySummary
	if exists {
		agg = *base
	} else {
		agg = DirectorySummary{Directory: dir}
	}

	for k, ds := range summaries {
		if k == dir {
			continue
		}
		if dir == "" {
			if k != "" {
				agg.Passed += ds.Passed
				agg.Failed += ds.Failed
				agg.Skipped += ds.Skipped
				agg.NotImplemented += ds.NotImplemented
				agg.RunnerError += ds.RunnerError
				agg.Crashed += ds.Crashed
				agg.Timeout += ds.Timeout
				agg.ParseError += ds.ParseError
				agg.Total += ds.Total
			}
		} else {
			prefix := dir + string(filepath.Separator)
			if strings.HasPrefix(k, prefix) {
				agg.Passed += ds.Passed
				agg.Failed += ds.Failed
				agg.Skipped += ds.Skipped
				agg.NotImplemented += ds.NotImplemented
				agg.RunnerError += ds.RunnerError
				agg.Crashed += ds.Crashed
				agg.Timeout += ds.Timeout
				agg.ParseError += ds.ParseError
				agg.Total += ds.Total
			}
		}
	}
	return agg
}
