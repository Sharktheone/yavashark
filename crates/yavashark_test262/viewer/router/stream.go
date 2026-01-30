package router

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
	"viewer/conf"
	"viewer/runhistory"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
	"yavashark_test262_runner/status"
	"yavashark_test262_runner/test"

	"github.com/gofiber/fiber/v2"
)

type StreamEvent struct {
	Type    string      `json:"type"`
	Data    interface{} `json:"data"`
	Message string      `json:"message,omitempty"`
}

type TestProgress struct {
	Path           string `json:"path"`
	Status         string `json:"status"`
	PreviousStatus string `json:"previousStatus,omitempty"`
	Message        string `json:"message,omitempty"`
	Duration       int64  `json:"duration,omitempty"`
}

type RunProgress struct {
	Total      int    `json:"total"`
	Completed  int    `json:"completed"`
	Passed     int    `json:"passed"`
	Failed     int    `json:"failed"`
	Skipped    int    `json:"skipped"`
	Crashed    int    `json:"crashed"`
	Timeout    int    `json:"timeout"`
	Phase      string `json:"phase"`
	CurrentDir string `json:"currentDir,omitempty"`
	RunID      string `json:"runId,omitempty"`
}

type RunMetadata struct {
	ID           string
	Path         string
	Profile      string
	StartedAt    time.Time
	Gained       int
	Lost         int
	BaselineType string
	BaselineRef  string
}

var (
	cancelMu   sync.Mutex
	cancelCtx  context.Context
	cancelFunc context.CancelFunc
)

func generateRunID() string {
	return runhistory.GenerateRunID()
}

func sendSSE(w *bufio.Writer, event StreamEvent) error {
	data, err := json.Marshal(event)
	if err != nil {
		return err
	}
	fmt.Fprintf(w, "data: %s\n\n", data)
	return w.Flush()
}

type buildOutputWriter struct {
	w           *bufio.Writer
	mu          sync.Mutex
	buffer      []byte
	outputLines []string
}

func newBuildOutputWriter(w *bufio.Writer) *buildOutputWriter {
	return &buildOutputWriter{
		w:           w,
		buffer:      make([]byte, 0, 1024),
		outputLines: make([]string, 0, 100),
	}
}

func (b *buildOutputWriter) GetOutputLines() []string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return append([]string(nil), b.outputLines...)
}

func (b *buildOutputWriter) Write(p []byte) (n int, err error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	b.buffer = append(b.buffer, p...)

	for {
		idx := -1
		for i, c := range b.buffer {
			if c == '\n' || c == '\r' {
				idx = i
				break
			}
		}

		if idx == -1 {
			if len(b.buffer) > 512 {
				line := string(b.buffer)
				b.buffer = b.buffer[:0]
				b.outputLines = append(b.outputLines, line)
				sendSSE(b.w, StreamEvent{
					Type:    "build_output",
					Message: line,
				})
			}
			break
		}

		line := string(b.buffer[:idx])
		if idx+1 < len(b.buffer) && b.buffer[idx] == '\r' && b.buffer[idx+1] == '\n' {
			b.buffer = b.buffer[idx+2:]
		} else {
			b.buffer = b.buffer[idx+1:]
		}

		if len(strings.TrimSpace(line)) > 0 {
			b.outputLines = append(b.outputLines, line)
			sendSSE(b.w, StreamEvent{
				Type:    "build_output",
				Message: line,
			})
		}
	}

	return len(p), nil
}

func (b *buildOutputWriter) Flush() {
	b.mu.Lock()
	defer b.mu.Unlock()

	if len(b.buffer) > 0 {
		line := string(b.buffer)
		b.buffer = b.buffer[:0]
		if len(strings.TrimSpace(line)) > 0 {
			b.outputLines = append(b.outputLines, line)
			sendSSE(b.w, StreamEvent{
				Type:    "build_output",
				Message: line,
			})
		}
	}
}

func rerunStream(c *fiber.Ctx) error {
	return runTestsWithStream(c, conf.TestRoot)
}

func rerunStreamPath(c *fiber.Ctx) error {
	path, err := filepath.Rel("/api/rerun-stream/", c.Path())
	if err != nil {
		return err
	}
	fullPath := filepath.Join(conf.TestRoot, path)
	return runTestsWithStream(c, fullPath)
}

func cancelRun(c *fiber.Ctx) error {
	cancelMu.Lock()
	defer cancelMu.Unlock()

	if cancelFunc != nil {
		cancelFunc()
		return c.JSON(fiber.Map{"status": "cancelled"})
	}
	return c.JSON(fiber.Map{"status": "no_run_active"})
}

func runTestsWithStream(c *fiber.Ctx, testPath string) error {
	rebuildFlag := c.Query("rebuild", "true") == "true"
	profile := c.Query("profile", "")
	baselineType := c.Query("baselineType", "current")
	baselineRef := c.Query("baselineRef", "")

	runMeta := RunMetadata{
		ID:           generateRunID(),
		Path:         testPath,
		Profile:      profile,
		StartedAt:    time.Now(),
		BaselineType: baselineType,
		BaselineRef:  baselineRef,
	}

	c.Set("Content-Type", "text/event-stream")
	c.Set("Cache-Control", "no-cache")
	c.Set("Connection", "keep-alive")
	c.Set("Transfer-Encoding", "chunked")

	c.Context().SetBodyStreamWriter(func(w *bufio.Writer) {
		if !conf.TestRunLock.TryLock() {
			sendSSE(w, StreamEvent{
				Type:    "error",
				Message: "Test is already running",
			})
			return
		}
		defer conf.TestRunLock.Unlock()

		cancelMu.Lock()
		cancelCtx, cancelFunc = context.WithCancel(context.Background())
		ctx := cancelCtx
		cancelMu.Unlock()

		defer func() {
			cancelMu.Lock()
			cancelFunc = nil
			cancelCtx = nil
			cancelMu.Unlock()
		}()

		sendSSE(w, StreamEvent{
			Type: "start",
			Data: map[string]string{
				"runId": runMeta.ID,
			},
		})

		if rebuildFlag {
			sendSSE(w, StreamEvent{
				Type: "progress",
				Data: RunProgress{Phase: "building", RunID: runMeta.ID},
			})

			select {
			case <-ctx.Done():
				sendSSE(w, StreamEvent{
					Type: "progress",
					Data: RunProgress{Phase: "cancelled", RunID: runMeta.ID},
				})
				return
			default:
			}

			buildConfig := build.Config{
				Rebuild:  true,
				Mode:     build.BuildModeRelease,
				Compiler: build.CompilerLLVM,
			}

			if profile != "" {
				profileConfig := loadProfileConfig(profile)
				if profileConfig != nil {
					if profileConfig.BuildMode != nil {
						mode, _ := build.ParseBuildMode(*profileConfig.BuildMode)
						buildConfig.Mode = mode
					}
					if profileConfig.BuildCompiler != nil {
						compiler, _ := build.ParseCompiler(*profileConfig.BuildCompiler)
						buildConfig.Compiler = compiler
					}
				}
			}

			buildWriter := newBuildOutputWriter(w)

			if err := build.RebuildEngineWithOutput(buildConfig, buildWriter, buildWriter); err != nil {
				buildWriter.Flush()
				sendSSE(w, StreamEvent{
					Type:    "error",
					Message: "Build failed: " + err.Error(),
				})
				return
			}
			buildWriter.Flush()

			select {
			case <-ctx.Done():
				sendSSE(w, StreamEvent{
					Type: "progress",
					Data: RunProgress{Phase: "cancelled", RunID: runMeta.ID},
				})
				return
			default:
			}

			sendSSE(w, StreamEvent{
				Type: "progress",
				Data: RunProgress{Phase: "counting", RunID: runMeta.ID},
			})

			runConfig := run.RunConfig{
				Workers:     conf.Workers,
				Skips:       true,
				Timings:     false,
				Timeout:     time.Duration(conf.ScriptTimeout) * time.Second,
				Interactive: false,
			}

			if profile != "" {
				profileConfig := loadProfileConfig(profile)
				if profileConfig != nil {
					if profileConfig.Workers != nil {
						runConfig.Workers = *profileConfig.Workers
					}
					if profileConfig.Timeout != nil {
						if d, err := time.ParseDuration(*profileConfig.Timeout); err == nil {
							runConfig.Timeout = d
						}
					}
					if profileConfig.NoSkip != nil {
						runConfig.Skips = !*profileConfig.NoSkip
					}
				}
			}

			streamingRun(ctx, w, testPath, runConfig, &runMeta, buildWriter.GetOutputLines())
		} else {
			select {
			case <-ctx.Done():
				sendSSE(w, StreamEvent{
					Type: "progress",
					Data: RunProgress{Phase: "cancelled", RunID: runMeta.ID},
				})
				return
			default:
			}

			sendSSE(w, StreamEvent{
				Type: "progress",
				Data: RunProgress{Phase: "counting", RunID: runMeta.ID},
			})

			runConfig := run.RunConfig{
				Workers:     conf.Workers,
				Skips:       true,
				Timings:     false,
				Timeout:     time.Duration(conf.ScriptTimeout) * time.Second,
				Interactive: false,
			}

			if profile != "" {
				profileConfig := loadProfileConfig(profile)
				if profileConfig != nil {
					if profileConfig.Workers != nil {
						runConfig.Workers = *profileConfig.Workers
					}
					if profileConfig.Timeout != nil {
						if d, err := time.ParseDuration(*profileConfig.Timeout); err == nil {
							runConfig.Timeout = d
						}
					}
					if profileConfig.NoSkip != nil {
						runConfig.Skips = !*profileConfig.NoSkip
					}
				}
			}

			streamingRun(ctx, w, testPath, runConfig, &runMeta, nil)
		}
	})

	return nil
}

type ProfileConfig struct {
	Workers       *int    `json:"workers,omitempty"`
	Timeout       *string `json:"timeout,omitempty"`
	NoSkip        *bool   `json:"noskip,omitempty"`
	BuildMode     *string `json:"build_mode,omitempty"`
	BuildCompiler *string `json:"build_compiler,omitempty"`
}

func loadProfileConfig(profileName string) *ProfileConfig {
	profilesPath := filepath.Join(conf.RunnerPath, "profiles.json")
	data, err := os.ReadFile(profilesPath)
	if err != nil {
		data, err = os.ReadFile("profiles.json")
		if err != nil {
			return nil
		}
	}

	var profilesConfig struct {
		Profiles map[string]ProfileConfig `json:"profiles"`
	}

	if err := json.Unmarshal(data, &profilesConfig); err != nil {
		return nil
	}

	if profile, ok := profilesConfig.Profiles[profileName]; ok {
		return &profile
	}
	return nil
}

func fetchResultsFromGitHub(commitHash string) ([]results.Result, error) {
	url := fmt.Sprintf("https://raw.githubusercontent.com/%s/%s/%s/results.json",
		DataRepoOwner, DataRepoName, commitHash)

	resp, err := http.Get(url)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch from GitHub: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == 404 {
		return nil, fmt.Errorf("results not found for commit %s", commitHash)
	}

	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("GitHub error: %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	var ciResults []results.CIResult
	if err := json.Unmarshal(body, &ciResults); err == nil && len(ciResults) > 0 {
		if ciResults[0].Path != "" {
			return results.ConvertResultsFromCI(ciResults), nil
		}
	}

	var fullResults []results.Result
	if err := json.Unmarshal(body, &fullResults); err != nil {
		return nil, fmt.Errorf("failed to parse results: %w", err)
	}

	return fullResults, nil
}

func streamingRun(ctx context.Context, w *bufio.Writer, testPath string, config run.RunConfig, runMeta *RunMetadata, buildOutput []string) {
	test.SetTimeout(config.Timeout)

	var prevResults []results.Result
	var baselineError error

	switch runMeta.BaselineType {
	case "commit":
		if runMeta.BaselineRef != "" {
			prevResults, baselineError = fetchResultsFromGitHub(runMeta.BaselineRef)
			if baselineError != nil {
				sendSSE(w, StreamEvent{
					Type:    "warning",
					Message: "Failed to fetch baseline from commit, falling back to current: " + baselineError.Error(),
				})
				prevResults, _ = results.LoadResults()
			}
		} else {
			prevResults, _ = results.LoadResults()
		}
	case "run":
		// TODO: Load results from a previous run (need to store run results separately)
		// For now, fall back to current results
		prevResults, _ = results.LoadResults()
	default:
		prevResults, _ = results.LoadResults()
	}

	prevResultsMap := make(map[string]status.Status)
	if prevResults != nil {
		for _, r := range prevResults {
			prevResultsMap[r.Path] = r.Status
		}
	}

	progress := RunProgress{
		Phase: "running",
		RunID: runMeta.ID,
	}

	gained := 0
	lost := 0
	changedTests := make([]ChangedTest, 0)

	fileInfo, err := os.Stat(testPath)
	if err != nil {
		sendSSE(w, StreamEvent{
			Type:    "error",
			Message: "Path not found: " + err.Error(),
		})
		return
	}

	var testPaths []string
	if fileInfo.IsDir() {
		filepath.Walk(testPath, func(p string, info os.FileInfo, err error) error {
			if err != nil || info.IsDir() {
				return nil
			}
			if filepath.Ext(p) == ".js" && !strings.Contains(p, "_FIXTURE") {
				testPaths = append(testPaths, p)
			}
			return nil
		})
	} else {
		if filepath.Ext(testPath) == ".js" {
			testPaths = []string{testPath}
		}
	}

	progress.Total = len(testPaths)
	sendSSE(w, StreamEvent{
		Type: "progress",
		Data: progress,
	})

	if len(testPaths) == 0 {
		sendSSE(w, StreamEvent{
			Type:    "error",
			Message: "No test files found",
		})
		return
	}

	jobs := make(chan string, config.Workers*2)
	resultsChan := make(chan results.Result, config.Workers*2)
	var wg sync.WaitGroup

	wg.Add(config.Workers)
	for i := 0; i < config.Workers; i++ {
		go func() {
			defer wg.Done()
			for {
				select {
				case <-ctx.Done():
					return
				case path, ok := <-jobs:
					if !ok {
						return
					}
					select {
					case <-ctx.Done():
						return
					default:
						res := test.RunTest(path, false)
						select {
						case resultsChan <- res:
						case <-ctx.Done():
							return
						}
					}
				}
			}
		}()
	}

	go func() {
		defer close(jobs)
		for _, path := range testPaths {
			select {
			case <-ctx.Done():
				return
			case jobs <- path:
			}
		}
	}()

	go func() {
		wg.Wait()
		close(resultsChan)
	}()

	allResults := make([]results.Result, 0, len(testPaths))
	cancelled := false

resultLoop:
	for {
		select {
		case <-ctx.Done():
			cancelled = true
			for {
				select {
				case res, ok := <-resultsChan:
					if !ok {
						break resultLoop
					}
					allResults = append(allResults, res)
				default:
					break resultLoop
				}
			}
		case res, ok := <-resultsChan:
			if !ok {
				break resultLoop
			}
			allResults = append(allResults, res)
			progress.Completed++

			switch res.Status {
			case status.PASS:
				progress.Passed++
			case status.FAIL:
				progress.Failed++
			case status.SKIP:
				progress.Skipped++
			case status.CRASH:
				progress.Crashed++
			case status.TIMEOUT:
				progress.Timeout++
			}

			var prevStatus string
			if prev, ok := prevResultsMap[res.Path]; ok {
				prevStatus = prev.String()
				if prev != status.PASS && res.Status == status.PASS {
					gained++
					changedTests = append(changedTests, ChangedTest{
						Path:      res.Path,
						OldStatus: prevStatus,
						NewStatus: res.Status.String(),
					})
				} else if prev == status.PASS && res.Status != status.PASS {
					lost++
					changedTests = append(changedTests, ChangedTest{
						Path:      res.Path,
						OldStatus: prevStatus,
						NewStatus: res.Status.String(),
					})
				}
			}

			sendSSE(w, StreamEvent{
				Type: "test",
				Data: TestProgress{
					Path:           res.Path,
					Status:         res.Status.String(),
					PreviousStatus: prevStatus,
					Message:        res.Msg,
					Duration:       res.Duration.Nanoseconds(),
				},
			})

			sendSSE(w, StreamEvent{
				Type: "progress",
				Data: progress,
			})
		}
	}

	saveRunToHistory := func(phase string) {
		relPath, _ := filepath.Rel(conf.TestRoot, testPath)
		if relPath == "." {
			relPath = ""
		}

		// Build before entries from prevResultsMap (filtered to tests we ran)
		beforeEntries := make([]runhistory.TestEntry, 0)
		for _, p := range testPaths {
			if prevStatus, ok := prevResultsMap[p]; ok {
				relTestPath, _ := filepath.Rel(conf.TestRoot, p)
				beforeEntries = append(beforeEntries, runhistory.TestEntry{
					Path:   relTestPath,
					Status: prevStatus.String(),
				})
			}
		}

		// Build after entries from allResults
		afterEntries := make([]runhistory.TestEntry, 0, len(allResults))
		for _, res := range allResults {
			relTestPath, _ := filepath.Rel(conf.TestRoot, res.Path)
			afterEntries = append(afterEntries, runhistory.TestEntry{
				Path:   relTestPath,
				Status: res.Status.String(),
			})
		}

		// Compute diff
		diff := runhistory.ComputeDiff(beforeEntries, afterEntries)

		completedAt := time.Now()
		duration := float64(completedAt.Sub(runMeta.StartedAt).Milliseconds())

		historyEntry := RunHistoryEntry{
			ID:           runMeta.ID,
			Path:         relPath,
			Profile:      runMeta.Profile,
			Source:       "stream",
			StartedAt:    runMeta.StartedAt,
			CompletedAt:  completedAt,
			Phase:        phase,
			Total:        progress.Total,
			Passed:       progress.Passed,
			Failed:       progress.Failed,
			Skipped:      progress.Skipped,
			Crashed:      progress.Crashed,
			Timeout:      progress.Timeout,
			Gained:       gained,
			Lost:         lost,
			ChangedTests: changedTests,
			BuildOutput:  buildOutput,
		}

		details := &runhistory.RunDetails{
			ID:       runMeta.ID,
			Before:   beforeEntries,
			After:    afterEntries,
			Diff:     diff,
			Duration: duration,
			Status:   phase,
			Options: runhistory.RunOptions{
				Dir: relPath,
			},
		}

		runhistory.SaveRun(historyEntry, details)
	}

	if cancelled {
		progress.Phase = "cancelled"
		sendSSE(w, StreamEvent{
			Type: "progress",
			Data: progress,
		})
		sendSSE(w, StreamEvent{
			Type:    "cancelled",
			Message: "Test run cancelled",
		})
		saveRunToHistory("cancelled")
		return
	}

	if len(allResults) > 0 {
		testResults := results.FromResults(allResults)
		if testPath != conf.TestRoot {
			existingResults, err := results.LoadResults()
			if err == nil && existingResults != nil {
				merged := testResults.MergeInto(existingResults)
				merged.Write()
			} else {
				testResults.Write()
			}
		} else {
			testResults.Write()
		}
	}

	progress.Phase = "complete"
	sendSSE(w, StreamEvent{
		Type: "progress",
		Data: progress,
	})

	sendSSE(w, StreamEvent{
		Type:    "complete",
		Message: "Test run complete",
	})

	saveRunToHistory("complete")
}

func countTestsInPath(path string) int {
	count := 0
	filepath.Walk(path, func(p string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		if filepath.Ext(p) == ".js" && !strings.Contains(p, "_FIXTURE") {
			count++
		}
		return nil
	})
	return count
}
