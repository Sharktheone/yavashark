package ci

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"time"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
)

func RunCi(tr *results.TestResults, repoPath string, historyOnly bool, diff bool, testRoot string) {
	if repoPath == "" {
		fmt.Println("CI mode enabled but no repository path specified via --repo")
		os.Exit(1)
	}

	overallSummary := Summary{
		Passed:         tr.Passed,
		Failed:         tr.Failed,
		Skipped:        tr.Skipped,
		NotImplemented: tr.NotImplemented,
		RunnerError:    tr.RunnerError,
		Crashed:        tr.Crashed,
		Timeout:        tr.Timeout,
		ParseError:     tr.ParseError,
		Total:          tr.Total,
		Timestamp:      time.Now().Unix(),
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
			overallSummary.Timestamp = time.Unix(commitEpoch, 0).Unix()
		}
	}

	if err := runCI(tr, overallSummary, repoPath, historyOnly, diff, testRoot); err != nil {
		panic(err)
	}
}

func runCI(testResults *results.TestResults, overall Summary, repo string, historyOnly bool, diff bool, root string) error {
	if err := generateHistoryFile(repo, overall); err != nil {
		return err
	}

	if historyOnly {
		log.Println("History file generated, skipping git commit")
		return nil
	}

	if diff {
		printCiDiff(filepath.Join(repo, "results.json"), testResults)
	}

	if err := results.WriteCIResultsPath(testResults.TestResults, filepath.Join(repo, "results.json"), root); err != nil {
		return err
	}

	resultsDir := filepath.Join(repo, "results")

	if err := os.RemoveAll(resultsDir); err != nil {
		return err
	}

	if err := os.MkdirAll(resultsDir, 0755); err != nil {
		return err
	}

	dirSummaries := map[string]*DirectorySummary{}

	for _, res := range testResults.TestResults {
		relPath, err := filepath.Rel(root, res.Path)
		if err != nil {
			relPath = res.Path
		}

		outFile := filepath.Join(resultsDir, relPath) + ".json"
		_ = os.MkdirAll(filepath.Dir(outFile), 0755)
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

	//cmd := exec.Command("git", "add", ".")
	//cmd.Dir = repo
	//if out, err := cmd.CombinedOutput(); err != nil {
	//	return fmt.Errorf("git add failed: %v, output: %s", err, out)
	//}
	//commitMsg := fmt.Sprintf("CI: Updated test results at %s", time.Now().Format(time.RFC3339))
	//cmd = exec.Command("git", "commit", "-m", commitMsg)
	//cmd.Dir = repo
	//if out, err := cmd.CombinedOutput(); err != nil {
	//	return fmt.Errorf("git commit failed: %v, output: %s", err, out)
	//}

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

func printCiDiff(path string, testResults *results.TestResults) {
	prev, _ := LoadPrevCi(path)
	if prev != nil {
		d, err := testResults.ComputeDiff(prev)
		if err != nil {
			return
		}

		d.PrintGrouped()
	}

	testResults.Compare(prev)
}
