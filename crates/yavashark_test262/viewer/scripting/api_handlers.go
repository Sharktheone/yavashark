package scripting

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
	"viewer/cache"
	"viewer/conf"
	"viewer/spec"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
	"yavashark_test262_runner/status"
)

// API handler types

type TestEntry struct {
	Path   string `json:"path"`
	Status string `json:"status"`
}

type TestOutput struct {
	Path     string  `json:"path"`
	Status   string  `json:"status"`
	Message  string  `json:"message"`
	Duration float64 `json:"duration"` // milliseconds
}

type QueryParams struct {
	Dir       string `json:"dir,omitempty"`
	Recursive bool   `json:"recursive,omitempty"`
	Status    any    `json:"status,omitempty"` // string or []string
	Query     string `json:"query,omitempty"`
}

type RerunOptions struct {
	Paths      []string `json:"paths,omitempty"`
	Dir        string   `json:"dir,omitempty"`
	FailedOnly bool     `json:"failedOnly,omitempty"`
	Rebuild    bool     `json:"rebuild,omitempty"`
}

type RerunResult struct {
	Before   []TestEntry `json:"before"`
	After    []TestEntry `json:"after"`
	Diff     DiffResult  `json:"diff"`
	Duration float64     `json:"duration"` // milliseconds
	Status   string      `json:"status"`   // "complete", "timeout", "cancelled"
}

type DiffResult struct {
	Gained  []TestEntry `json:"gained"`
	Lost    []TestEntry `json:"lost"`
	Changed []TestEntry `json:"changed"`
}

// In-memory code edits (not persisted to disk)
var (
	testCodeEdits    = make(map[string]string)
	harnessCodeEdits = make(map[string]string)
)

// HandleAPICall processes an API call from the TypeScript runtime
func HandleAPICall(method string, params json.RawMessage) (any, error) {
	switch method {
	// tests.*
	case "tests.query":
		return handleTestsQuery(params)
	case "tests.getStatus":
		return handleTestsGetStatus(params)
	case "tests.getOutput":
		return handleTestsGetOutput(params)
	case "tests.getCode":
		return handleTestsGetCode(params)
	case "tests.setCode":
		return handleTestsSetCode(params)

	// harness.*
	case "harness.getCode":
		return handleHarnessGetCode(params)
	case "harness.listForTest":
		return handleHarnessListForTest(params)
	case "harness.getForTest":
		return handleHarnessGetForTest(params)
	case "harness.setCode":
		return handleHarnessSetCode(params)

	// spec.*
	case "spec.get":
		return handleSpecGet(params)
	case "spec.search":
		return handleSpecSearch(params)
	case "spec.forIntrinsic":
		return handleSpecForIntrinsic(params)

	// runner.*
	case "runner.rerun":
		return handleRunnerRerun(params)
	case "runner.rerunAsync":
		return handleRunnerRerunAsync(params)
	case "runner.getJob":
		return handleRunnerGetJob(params)
	case "runner.cancelJob":
		return handleRunnerCancelJob(params)

	// output.*
	case "output.setMaxChars":
		return handleOutputSetMaxChars(params)
	case "output.getMaxChars":
		return handleOutputGetMaxChars(params)
	case "output.setOffset":
		return handleOutputSetOffset(params)
	case "output.getOffset":
		return handleOutputGetOffset(params)
	case "output.setMode":
		return handleOutputSetMode(params)
	case "output.getMode":
		return handleOutputGetMode(params)
	case "output.configure":
		return handleOutputConfigure(params)
	case "output.getConfig":
		return handleOutputGetConfig(params)
	case "output.getLastLength":
		return handleOutputGetLastLength(params)

	default:
		return nil, fmt.Errorf("unknown method: %s", method)
	}
}

// tests.* handlers

func handleTestsQuery(params json.RawMessage) (any, error) {
	var p QueryParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	resultsMap, err := cache.GetResultsIndex()
	if err != nil {
		return nil, err
	}

	var entries []TestEntry

	for path, result := range *resultsMap {
		// Apply directory filter
		if p.Dir != "" {
			relPath, err := filepath.Rel(conf.TestRoot, path)
			if err != nil {
				continue
			}
			if p.Recursive {
				if !strings.HasPrefix(relPath, p.Dir) {
					continue
				}
			} else {
				dir := filepath.Dir(relPath)
				if dir != p.Dir {
					continue
				}
			}
		}

		// Apply status filter
		if p.Status != nil {
			statusStr := result.Status.String()
			match := false

			switch s := p.Status.(type) {
			case string:
				match = statusStr == s
			case []interface{}:
				for _, v := range s {
					if vs, ok := v.(string); ok && vs == statusStr {
						match = true
						break
					}
				}
			}

			if !match {
				continue
			}
		}

		// Apply search query
		if p.Query != "" {
			if !strings.Contains(strings.ToLower(path), strings.ToLower(p.Query)) {
				continue
			}
		}

		relPath, err := filepath.Rel(conf.TestRoot, path)
		if err != nil {
			relPath = path
		}

		entries = append(entries, TestEntry{
			Path:   relPath,
			Status: result.Status.String(),
		})
	}

	return entries, nil
}

func handleTestsGetStatus(params json.RawMessage) (any, error) {
	var p struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	resultsMap, err := cache.GetResultsIndex()
	if err != nil {
		return nil, err
	}

	fullPath := filepath.Join(conf.TestRoot, p.Path)
	if result, ok := (*resultsMap)[fullPath]; ok {
		return TestEntry{
			Path:   p.Path,
			Status: result.Status.String(),
		}, nil
	}

	return nil, fmt.Errorf("test not found: %s", p.Path)
}

func handleTestsGetOutput(params json.RawMessage) (any, error) {
	var p struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	resultsMap, err := cache.GetResultsIndex()
	if err != nil {
		return nil, err
	}

	fullPath := filepath.Join(conf.TestRoot, p.Path)
	if result, ok := (*resultsMap)[fullPath]; ok {
		return TestOutput{
			Path:     p.Path,
			Status:   result.Status.String(),
			Message:  result.Msg,
			Duration: float64(result.Duration.Milliseconds()),
		}, nil
	}

	return nil, fmt.Errorf("test not found: %s", p.Path)
}

func handleTestsGetCode(params json.RawMessage) (any, error) {
	var p struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	// Check for in-memory edits first
	if code, ok := testCodeEdits[p.Path]; ok {
		return code, nil
	}

	fullPath := filepath.Join(conf.TestRoot, p.Path)
	code, err := os.ReadFile(fullPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read test: %w", err)
	}

	return string(code), nil
}

func handleTestsSetCode(params json.RawMessage) (any, error) {
	var p struct {
		Path string `json:"path"`
		Code string `json:"code"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	// Store in memory only (not persisted to disk)
	testCodeEdits[p.Path] = p.Code
	return nil, nil
}

// harness.* handlers

func handleHarnessGetCode(params json.RawMessage) (any, error) {
	var p struct {
		Name string `json:"name"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	// Check for in-memory edits first
	if code, ok := harnessCodeEdits[p.Name]; ok {
		return code, nil
	}

	// Try to find harness file
	harnessPath := filepath.Join(conf.TestRoot, "..", "harness", p.Name)
	code, err := os.ReadFile(harnessPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read harness file: %w", err)
	}

	return string(code), nil
}

func handleHarnessListForTest(params json.RawMessage) (any, error) {
	var p struct {
		TestPath string `json:"testPath"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	// Read the test file and parse its metadata for includes
	fullPath := filepath.Join(conf.TestRoot, p.TestPath)
	content, err := os.ReadFile(fullPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read test: %w", err)
	}

	// Parse includes from test262 frontmatter
	includes := parseTest262Includes(string(content))
	return includes, nil
}

func handleHarnessGetForTest(params json.RawMessage) (any, error) {
	var p struct {
		TestPath string `json:"testPath"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	includes, err := handleHarnessListForTest(params)
	if err != nil {
		return nil, err
	}

	result := make(map[string]string)
	for _, name := range includes.([]string) {
		code, err := handleHarnessGetCode(json.RawMessage(fmt.Sprintf(`{"name":"%s"}`, name)))
		if err == nil {
			result[name] = code.(string)
		}
	}

	return result, nil
}

func handleHarnessSetCode(params json.RawMessage) (any, error) {
	var p struct {
		Name string `json:"name"`
		Code string `json:"code"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	// Store in memory only
	harnessCodeEdits[p.Name] = p.Code
	return nil, nil
}

// spec.* handlers

func handleSpecGet(params json.RawMessage) (any, error) {
	var p struct {
		Section string `json:"section"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	provider := spec.GetProvider()
	content, err := provider.GetSpec(p.Section)
	if err != nil {
		return nil, err
	}

	return map[string]string{
		"section": p.Section,
		"content": content,
	}, nil
}

func handleSpecSearch(params json.RawMessage) (any, error) {
	var p struct {
		Query string `json:"query"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	provider := spec.GetProvider()
	// SearchSpec searches content (like mcp262's SearchSections)
	sections, err := provider.SearchSpec(p.Query)
	if err != nil {
		return nil, err
	}

	// Return as array of match objects for consistency with TypeScript API
	type SpecMatch struct {
		Section string `json:"section"`
	}
	matches := make([]SpecMatch, len(sections))
	for i, s := range sections {
		matches[i] = SpecMatch{Section: s}
	}

	return matches, nil
}

func handleSpecForIntrinsic(params json.RawMessage) (any, error) {
	var p struct {
		Name string `json:"name"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}

	provider := spec.GetProvider()
	content, err := provider.SpecForIntrinsic(p.Name)
	if err != nil {
		return nil, err
	}

	return map[string]string{
		"intrinsic": p.Name,
		"content":   content,
	}, nil
}

// runner.* handlers

func handleRunnerRerun(params json.RawMessage) (any, error) {
	var opts RerunOptions
	if err := json.Unmarshal(params, &opts); err != nil {
		return nil, err
	}

	// Rebuild if requested
	if opts.Rebuild {
		if err := build.RebuildEngine(build.Config{
			Rebuild:  true,
			Mode:     build.BuildModeRelease,
			Compiler: build.CompilerLLVM,
		}); err != nil {
			return nil, fmt.Errorf("build failed: %w", err)
		}
	}

	// Get before state
	resultsMap, err := cache.GetResultsIndex()
	if err != nil {
		return nil, err
	}

	// Determine what to run
	var testDir string
	if opts.Dir != "" {
		testDir = filepath.Join(conf.TestRoot, opts.Dir)
	} else {
		testDir = conf.TestRoot
	}

	// Collect before entries
	var beforeEntries []TestEntry
	for path, result := range *resultsMap {
		relPath, _ := filepath.Rel(conf.TestRoot, path)
		if opts.Dir != "" && !strings.HasPrefix(relPath, opts.Dir) {
			continue
		}
		if opts.FailedOnly && result.Status == status.PASS {
			continue
		}
		beforeEntries = append(beforeEntries, TestEntry{
			Path:   relPath,
			Status: result.Status.String(),
		})
	}

	startTime := time.Now()

	// Run tests
	runConfig := run.RunConfig{
		Workers:     conf.Workers,
		Skips:       true,
		Timings:     false,
		Timeout:     30 * time.Second,
		Interactive: false,
	}

	newResults, _ := run.TestsInDir(testDir, runConfig)

	// Merge results
	existingResults, err := results.LoadResults()
	if err == nil && existingResults != nil {
		merged := newResults.MergeInto(existingResults)
		merged.Write()
	} else {
		newResults.Write()
	}

	duration := time.Since(startTime)

	// Refresh cache and get after state
	cache.InitWithCurrent()
	resultsMap, _ = cache.GetResultsIndex()

	// Collect after entries
	var afterEntries []TestEntry
	for path, result := range *resultsMap {
		relPath, _ := filepath.Rel(conf.TestRoot, path)
		if opts.Dir != "" && !strings.HasPrefix(relPath, opts.Dir) {
			continue
		}
		afterEntries = append(afterEntries, TestEntry{
			Path:   relPath,
			Status: result.Status.String(),
		})
	}

	// Compute diff
	diff := computeDiff(beforeEntries, afterEntries)

	return RerunResult{
		Before:   beforeEntries,
		After:    afterEntries,
		Diff:     diff,
		Duration: float64(duration.Milliseconds()),
		Status:   "complete",
	}, nil
}

func handleRunnerRerunAsync(params json.RawMessage) (any, error) {
	// TODO: Implement async job system
	return nil, fmt.Errorf("async rerun not yet implemented")
}

func handleRunnerGetJob(params json.RawMessage) (any, error) {
	// TODO: Implement async job system
	return nil, fmt.Errorf("job system not yet implemented")
}

func handleRunnerCancelJob(params json.RawMessage) (any, error) {
	// TODO: Implement async job system
	return nil, fmt.Errorf("job system not yet implemented")
}

// output.* handlers

func handleOutputSetMaxChars(params json.RawMessage) (any, error) {
	var p struct {
		MaxChars int `json:"maxChars"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}
	if p.MaxChars < 0 {
		return nil, fmt.Errorf("maxChars must be >= 0")
	}
	conf.MaxOutputChars = p.MaxChars
	return nil, nil
}

func handleOutputGetMaxChars(params json.RawMessage) (any, error) {
	return map[string]any{
		"maxChars":  conf.MaxOutputChars,
		"unlimited": conf.MaxOutputChars == 0,
	}, nil
}

func handleOutputSetOffset(params json.RawMessage) (any, error) {
	var p struct {
		Offset int `json:"offset"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}
	if p.Offset < 0 {
		return nil, fmt.Errorf("offset must be >= 0")
	}
	conf.OutputOffset = p.Offset
	return nil, nil
}

func handleOutputGetOffset(params json.RawMessage) (any, error) {
	return map[string]int{"offset": conf.OutputOffset}, nil
}

func handleOutputSetMode(params json.RawMessage) (any, error) {
	var p struct {
		Mode string `json:"mode"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}
	if p.Mode != "head" && p.Mode != "tail" {
		return nil, fmt.Errorf("mode must be 'head' or 'tail'")
	}
	conf.OutputMode = p.Mode
	return nil, nil
}

func handleOutputGetMode(params json.RawMessage) (any, error) {
	return map[string]string{"mode": conf.OutputMode}, nil
}

func handleOutputConfigure(params json.RawMessage) (any, error) {
	var p struct {
		MaxChars *int    `json:"maxChars,omitempty"`
		Offset   *int    `json:"offset,omitempty"`
		Mode     *string `json:"mode,omitempty"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, err
	}
	if p.MaxChars != nil {
		if *p.MaxChars < 0 {
			return nil, fmt.Errorf("maxChars must be >= 0")
		}
		conf.MaxOutputChars = *p.MaxChars
	}
	if p.Offset != nil {
		if *p.Offset < 0 {
			return nil, fmt.Errorf("offset must be >= 0")
		}
		conf.OutputOffset = *p.Offset
	}
	if p.Mode != nil {
		if *p.Mode != "head" && *p.Mode != "tail" {
			return nil, fmt.Errorf("mode must be 'head' or 'tail'")
		}
		conf.OutputMode = *p.Mode
	}
	return nil, nil
}

func handleOutputGetConfig(params json.RawMessage) (any, error) {
	return map[string]any{
		"maxChars":  conf.MaxOutputChars,
		"offset":    conf.OutputOffset,
		"mode":      conf.OutputMode,
		"unlimited": conf.MaxOutputChars == 0,
	}, nil
}

func handleOutputGetLastLength(params json.RawMessage) (any, error) {
	return map[string]int{"length": conf.LastOutputLength}, nil
}

// Helper functions

func parseTest262Includes(content string) []string {
	// Look for includes in YAML frontmatter
	// Format:
	// /*---
	// includes: [file1.js, file2.js]
	// ---*/

	var includes []string

	// Simple parsing - look for includes: line
	lines := strings.Split(content, "\n")
	inFrontmatter := false

	for _, line := range lines {
		line = strings.TrimSpace(line)

		if line == "/*---" {
			inFrontmatter = true
			continue
		}
		if line == "---*/" {
			break
		}

		if inFrontmatter && strings.HasPrefix(line, "includes:") {
			// Parse includes: [file1.js, file2.js]
			rest := strings.TrimPrefix(line, "includes:")
			rest = strings.TrimSpace(rest)
			rest = strings.Trim(rest, "[]")

			parts := strings.Split(rest, ",")
			for _, part := range parts {
				part = strings.TrimSpace(part)
				if part != "" {
					includes = append(includes, part)
				}
			}
		}
	}

	// Always include assert.js and sta.js by default
	hasAssert := false
	hasSta := false
	for _, inc := range includes {
		if inc == "assert.js" {
			hasAssert = true
		}
		if inc == "sta.js" {
			hasSta = true
		}
	}
	if !hasAssert {
		includes = append([]string{"assert.js"}, includes...)
	}
	if !hasSta {
		includes = append([]string{"sta.js"}, includes...)
	}

	return includes
}

func computeDiff(before, after []TestEntry) DiffResult {
	beforeMap := make(map[string]string)
	for _, e := range before {
		beforeMap[e.Path] = e.Status
	}

	afterMap := make(map[string]string)
	for _, e := range after {
		afterMap[e.Path] = e.Status
	}

	var gained, lost, changed []TestEntry

	for _, e := range after {
		beforeStatus, existed := beforeMap[e.Path]
		if !existed {
			continue
		}

		if beforeStatus != e.Status {
			changed = append(changed, e)

			if e.Status == "PASS" && beforeStatus != "PASS" {
				gained = append(gained, e)
			} else if e.Status != "PASS" && beforeStatus == "PASS" {
				lost = append(lost, e)
			}
		}
	}

	return DiffResult{
		Gained:  gained,
		Lost:    lost,
		Changed: changed,
	}
}

// ResetEdits clears all in-memory code edits
func ResetEdits() {
	testCodeEdits = make(map[string]string)
	harnessCodeEdits = make(map[string]string)
}
