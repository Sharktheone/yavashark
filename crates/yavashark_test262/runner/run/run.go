package run

import (
	"bytes"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/progress"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/scheduler"
	"yavashark_test262_runner/status"
)

var SKIP = []string{"intl402", "staging"}

type RunConfig struct {
	Prepared        *Prepared
	Workers         int
	RiskyWorkers    int
	RiskyCPUs       int
	CPUCount        int
	Selection       string
	Coverage        float64
	CalibrationPath string
	CalibrationKey  string
	HistoryPath     string
	Engine          string
	ReportPath      string
	Skips           bool
	Timings         bool
	Timeout         time.Duration
	Interactive     bool
	FailedOnly      bool
}

// Discover is shared by regular execution and calibration. Overlapping paths
// are deduplicated; fixture/non-JS files never become jobs.
func Discover(paths []string, skip bool) (tests, skipped, errors []string) {
	seen := map[string]bool{}
	for _, root := range paths {
		err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
			if err != nil {
				errors = append(errors, path)
				return nil
			}
			if d.IsDir() {
				return nil
			}
			if strings.Contains(path, "_FIXTURE") || !isTestFile(path) {
				return nil
			}
			abs, err := filepath.Abs(path)
			if err != nil {
				return err
			}
			if seen[abs] {
				return nil
			}
			seen[abs] = true
			k := calibration.Key(path)
			if skip {
				for _, prefix := range SKIP {
					if strings.HasPrefix(k, prefix+"/") {
						skipped = append(skipped, path)
						return nil
					}
				}
			}
			tests = append(tests, path)
			return nil
		})
		if err != nil {
			errors = append(errors, root)
		}
	}
	sort.Strings(tests)
	sort.Strings(skipped)
	return
}
func TestsInDir(root string, config RunConfig) (*results.TestResults, progress.Summary) {
	tr, s, errors := TestSpecificPaths([]string{root}, config)
	for _, p := range errors {
		log.Printf("Could not read test path: %s", p)
	}
	return tr, s
}
func TestSpecificPaths(paths []string, config RunConfig) (*results.TestResults, progress.Summary, []string) {
	tr, s, errs, err := runPaths(paths, config)
	if err != nil {
		log.Printf("Runner error: %v", err)
		tr = results.New(1)
		tr.Add(results.Result{Status: status.RUNNER_ERROR, Msg: err.Error()})
		s = progress.Summary{RunnerError: 1, Total: 1}
		errs = append(errs, err.Error())
	}
	return tr, s, errs
}
func runPaths(paths []string, config RunConfig) (*results.TestResults, progress.Summary, []string, error) {
	inputs, err := preparedInputs(paths, config)
	if err != nil {
		return nil, progress.Summary{}, nil, err
	}
	tests, skipped := append([]string(nil), inputs.tests...), inputs.skipped
	var errors []string
	config.Prepared = inputs
	store, env, opts, err := resolve(config)
	if err != nil {
		return nil, progress.Summary{}, errors, err
	}
	history := inputs.history

	prev := map[string]status.Status{}
	for _, p := range tests {
		if r, ok := history[calibration.Key(p)]; ok {
			prev[p] = r.Status
		}
	}
	if config.FailedOnly {
		filtered := tests[:0]
		for _, p := range tests {
			if s, ok := prev[p]; !ok || s != status.PASS {
				filtered = append(filtered, p)
			}
		}
		tests = filtered
	}
	selection := config.Selection
	if selection == "" {
		selection = "full"
	}
	coverage := config.Coverage
	if coverage == 0 {
		coverage = 100
	}
	plan := scheduler.Make(tests, env, history, selection, coverage)
	if config.RiskyWorkers == 0 && env.Settings == nil {
		hangs := 0
		for _, j := range plan.Risky {
			if j.TimedOut {
				hangs++
			}
		}
		opts.Settings.RiskyWorkers = max(opts.Settings.RiskyWorkers, min(hangs, 128))
	}
	n := len(tests) - plan.Excluded + len(skipped)
	log.Printf("Selection %s: %d/%d runnable tests; quick range %d, excluded %d, unknown costs %d", selection, len(tests)-plan.Excluded, len(tests), plan.QuickEligible, plan.Excluded, plan.Unknown)
	log.Printf("Queues: fast=%d medium=%d slow=%d risky=%d; settings=%+v", len(plan.Fast), len(plan.Medium), len(plan.Slow), len(plan.Risky), opts.Settings)
	tracker := progress.NewProgressTracker(uint32(n), config.Interactive, prev)
	tr := results.New(uint32(n))
	emit := func(r results.Result) { tr.Add(r); tracker.Add(r.Status, r.Path) }
	stats := Execute(plan, opts, emit)
	for _, p := range skipped {
		emit(results.Result{Path: p, Status: status.SKIP, Msg: "excluded test directory"})
	}
	summary := tracker.Finish()
	env.Observe(tr.TestResults)
	file := config.CalibrationPath
	if file == "" {
		file = "runner-calibration.json"
	}
	if err = calibration.Save(file, store); err != nil {
		return tr, summary, errors, err
	}
	if config.ReportPath != "" {
		// Timing artifacts do not need the engine's potentially large diagnostics;
		// those remain in the ordinary conformance results.
		type measurement struct {
			Path     string        `json:"path"`
			Status   status.Status `json:"status"`
			CPUTime  time.Duration `json:"cpu_time"`
			Duration time.Duration `json:"duration"`
			MemoryKB uint64        `json:"memory_kb"`
		}
		measurements := make([]measurement, len(tr.TestResults))
		for i, r := range tr.TestResults {
			measurements[i] = measurement{r.Path, r.Status, r.CPUTime, r.Duration, r.MemoryKB}
		}
		report := struct {
			Selection     string         `json:"selection"`
			Coverage      float64        `json:"coverage"`
			Discovered    int            `json:"discovered"`
			QuickEligible int            `json:"quick_eligible"`
			Excluded      int            `json:"excluded"`
			Unknown       int            `json:"unknown"`
			Stats         ExecutionStats `json:"execution"`
			Results       []measurement  `json:"results"`
		}{selection, coverage, len(tests), plan.QuickEligible, plan.Excluded, plan.Unknown, stats, measurements}
		if err = calibration.Save(config.ReportPath, report); err != nil {
			return tr, summary, errors, err
		}
	}
	log.Printf("Finished %d tests in %s (not selected: %d); CPU masks normal=%v risky=%v", tr.Total, stats.Duration, plan.Excluded, stats.NormalCPUs, stats.RiskyCPUs)
	return tr, summary, errors, nil
}

// Some upstream tests have extensionless names. Recognize their Test262 header
// without accidentally scheduling result JSON or other repository artifacts.
func isTestFile(path string) bool {
	if strings.HasSuffix(path, ".js") {
		return true
	}
	if filepath.Ext(path) != "" {
		return false
	}
	f, err := os.Open(path)
	if err != nil {
		return false
	}
	defer f.Close()
	var header [4096]byte
	n, _ := f.Read(header[:])
	return bytes.Contains(header[:n], []byte("/*---"))
}
