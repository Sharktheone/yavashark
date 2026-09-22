package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"math"
	"os"
	"path/filepath"
	"strings"
	"time"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/run"
)

const (
	DEFAULT_PROFILE_FILE = "profiles.json"
)

type Config struct {
	prepared        *run.Prepared
	RiskyWorkers    int     `json:"risky_workers"`
	RiskyCPUs       int     `json:"risky_cpus"`
	CPUCount        int     `json:"cpus"`
	Selection       string  `json:"selection"`
	Coverage        float64 `json:"coverage"`
	CalibrationPath string  `json:"calibration"`
	CalibrationKey  string  `json:"calibration_key"`
	HistoryPath     string  `json:"history"`
	Engine          string  `json:"engine"`
	ReportPath      string  `json:"report"`
	Bench           bool    `json:"bench"`
	BenchSamples    int     `json:"bench_samples"`
	BenchRepeats    int     `json:"bench_repeats"`
	BenchOutput     string  `json:"bench_output"`
	ProbeTimeout    string  `json:"probe_timeout"`
	Output          string  `json:"output"`

	CI          bool          `json:"ci"`
	RepoPath    string        `json:"repo_path"`
	HistoryOnly bool          `json:"history_only"`
	Workers     int           `json:"workers"`
	TestRootDir string        `json:"test_root_dir"`
	Diff        bool          `json:"diff"`
	DiffFilter  string        `json:"diff_filter"`
	TestDir     string        `json:"test_dir"`
	Skips       bool          `json:"skips"`
	Timings     bool          `json:"timings"`
	Timeout     time.Duration `json:"timeout"`
	Interactive bool          `json:"interactive"`
	ShowStats   bool          `json:"show_stats"`
	Verbose     bool          `json:"verbose"`

	Rebuild       bool            `json:"rebuild"`
	BuildMode     build.BuildMode `json:"build_mode"`
	BuildCompiler build.Compiler  `json:"build_compiler"`

	FilterPath string `json:"-"`
}

type ProfileConfig struct {
	Profiles map[string]Profile `json:"profiles"`
}

type Profile struct {
	RiskyWorkers    *int     `json:"risky_workers,omitempty"`
	RiskyCPUs       *int     `json:"risky_cpus,omitempty"`
	CPUCount        *int     `json:"cpus,omitempty"`
	Selection       *string  `json:"selection,omitempty"`
	Coverage        *float64 `json:"coverage,omitempty"`
	CalibrationPath *string  `json:"calibration,omitempty"`
	CalibrationKey  *string  `json:"calibration_key,omitempty"`
	HistoryPath     *string  `json:"history,omitempty"`
	Engine          *string  `json:"engine,omitempty"`
	ReportPath      *string  `json:"report,omitempty"`
	Bench           *bool    `json:"bench,omitempty"`
	BenchSamples    *int     `json:"bench_samples,omitempty"`
	BenchRepeats    *int     `json:"bench_repeats,omitempty"`
	BenchOutput     *string  `json:"bench_output,omitempty"`
	ProbeTimeout    *string  `json:"probe_timeout,omitempty"`
	Output          *string  `json:"output,omitempty"`

	CI          *bool   `json:"ci,omitempty"`
	RepoPath    *string `json:"repo_path,omitempty"`
	HistoryOnly *bool   `json:"history_only,omitempty"`
	Workers     *int    `json:"workers,omitempty"`
	TestRootDir *string `json:"test_root,omitempty"`
	Diff        *bool   `json:"diff,omitempty"`
	DiffFilter  *string `json:"diff_filter,omitempty"`
	TestDir     *string `json:"test_dir,omitempty"`
	NoSkip      *bool   `json:"noskip,omitempty"`
	Timings     *bool   `json:"timings,omitempty"`
	Timeout     *string `json:"timeout,omitempty"`
	Interactive *bool   `json:"interactive,omitempty"`
	ShowStats   *bool   `json:"show_stats,omitempty"`
	Verbose     *bool   `json:"verbose,omitempty"`

	Rebuild       *bool   `json:"rebuild,omitempty"`
	BuildMode     *string `json:"build_mode,omitempty"`
	BuildCompiler *string `json:"build_compiler,omitempty"`
}

func NewConfig() *Config {
	return &Config{
		RiskyWorkers:    0,
		RiskyCPUs:       0,
		CPUCount:        0,
		Selection:       "full",
		Coverage:        100,
		CalibrationPath: "",
		CalibrationKey:  "",
		HistoryPath:     "",
		Engine:          "",
		ReportPath:      "",
		Bench:           false,
		BenchSamples:    128,
		BenchRepeats:    2,
		BenchOutput:     "runner-bench.json",
		ProbeTimeout:    "2s",
		Output:          "",

		CI:          false,
		RepoPath:    "",
		HistoryOnly: false,
		Workers:     0,
		TestRootDir: DEFAULT_TEST_ROOT,
		Diff:        true,
		DiffFilter:  "",
		TestDir:     "",
		Skips:       true,
		Timings:     false,
		Timeout:     30 * time.Second,
		Interactive: false,
		ShowStats:   false,
		Verbose:     false,

		Rebuild:       false,
		BuildMode:     build.BuildModeRelease,
		BuildCompiler: build.CompilerLLVM,
	}
}

func LoadConfig() *Config {
	config := NewConfig()

	profileFile := flag.String("profiles", DEFAULT_PROFILE_FILE, "Path to JSON profiles file")
	profile := flag.String("p", "", "Profile name to load from profiles file")
	ciEnabled := flag.Bool("ci", config.CI, "Enable CI mode to commit results")
	repoPath := flag.String("repo", config.RepoPath, "Path to external repository for CI results")
	historyOnly := flag.Bool("history-only", config.HistoryOnly, "Only generate the history file (skip git commit)")
	workers := flag.Int("workers", config.Workers, "Normal workers in every phase; 0 uses calibrated per-phase counts")
	testRootDir := flag.String("test_root", config.TestRootDir, "Path to test root directory")
	diff := flag.Bool("diff", config.Diff, "Diff to use for CI results")
	diffFilter := flag.String("dfilter", config.DiffFilter, "Diff filter to use for CI results")
	testdir := flag.String("testdir", config.TestDir, "Path in the test directory")
	noskip := flag.Bool("noskip", false, "Disable skipping of certain test directories")
	timings := flag.Bool("timings", false, "Attempt to parse timings from test output (if enabled)")
	timeout := flag.Duration("timeout", config.Timeout, "Timeout for each test (e.g., 30s, 1m)")
	interactive := flag.Bool("i", false, "Enable interactive TUI mode")
	showStats := flag.Bool("stats", false, "Show memory and timing statistics")
	verbose := flag.Bool("v", false, "Show verbose output (detailed results)")

	rebuild := flag.Bool("rebuild", config.Rebuild, "Rebuild the engine before running tests")
	buildMode := flag.String("build-mode", string(config.BuildMode), "Build mode: debug or release")
	buildCompiler := flag.String("compiler", string(config.BuildCompiler), "Compiler backend: llvm or cranelift")

	optRiskyWorkers := flag.Int("risky-workers", config.RiskyWorkers, "Concurrent risky processes; 0 uses calibration")
	optRiskyCPUs := flag.Int("risky-cpus", config.RiskyCPUs, "CPUs reserved for risky tests; 0 uses calibration")
	optCPUCount := flag.Int("cpus", config.CPUCount, "Engine CPU budget; 0 uses all available CPUs")
	optSelection := flag.String("selection", config.Selection, "Test selection: full or quick")
	optCoverage := flag.Float64("coverage", config.Coverage, "Percentage within quick range only (0,100]")
	optCalibrationPath := flag.String("calibration", config.CalibrationPath, "Calibration JSON; CI defaults to REPO/runner-calibration.json")
	optCalibrationKey := flag.String("calibration-key", config.CalibrationKey, "Hardware-specific calibration key")
	optHistoryPath := flag.String("history", config.HistoryPath, "Prior detailed or compact CI results")
	optEngine := flag.String("engine", config.Engine, "Engine executable path")
	optReportPath := flag.String("report", config.ReportPath, "Detailed timing/results report path")
	optBench := flag.Bool("bench", config.Bench, "Benchmark scheduler settings without a conformance run")
	optBenchSamples := flag.Int("bench-samples", config.BenchSamples, "Sample size per fast/medium phase")
	optBenchRepeats := flag.Int("bench-repeats", config.BenchRepeats, "Measurements per benchmark configuration")
	optBenchOutput := flag.String("bench-output", config.BenchOutput, "Benchmark trial report")
	optProbeTimeout := flag.String("probe-timeout", config.ProbeTimeout, "Per-test timeout for calibration probes only")
	optOutput := flag.String("output", config.Output, "Results output path (quick defaults to results-quick.json)")

	flag.Parse()

	// Load profile if specified
	if *profile != "" {
		log.Printf("Loading profile '%s' from '%s'", *profile, *profileFile)
		if err := loadProfile(*profileFile, *profile, config); err != nil {
			log.Printf("Warning: Failed to load profile '%s': %v", *profile, err)
		}
	}

	// Override with command-line flags (flags take precedence over profile)
	flag.Visit(func(f *flag.Flag) {
		switch f.Name {
		case "risky-workers":
			config.RiskyWorkers = *optRiskyWorkers
		case "risky-cpus":
			config.RiskyCPUs = *optRiskyCPUs
		case "cpus":
			config.CPUCount = *optCPUCount
		case "selection":
			config.Selection = *optSelection
		case "coverage":
			config.Coverage = *optCoverage
		case "calibration":
			config.CalibrationPath = *optCalibrationPath
		case "calibration-key":
			config.CalibrationKey = *optCalibrationKey
		case "history":
			config.HistoryPath = *optHistoryPath
		case "engine":
			config.Engine = *optEngine
		case "report":
			config.ReportPath = *optReportPath
		case "bench":
			config.Bench = *optBench
		case "bench-samples":
			config.BenchSamples = *optBenchSamples
		case "bench-repeats":
			config.BenchRepeats = *optBenchRepeats
		case "bench-output":
			config.BenchOutput = *optBenchOutput
		case "probe-timeout":
			config.ProbeTimeout = *optProbeTimeout
		case "output":
			config.Output = *optOutput

		case "ci":
			config.CI = *ciEnabled
		case "repo":
			config.RepoPath = *repoPath
		case "history-only":
			config.HistoryOnly = *historyOnly
		case "workers":
			config.Workers = *workers
		case "test_root":
			config.TestRootDir = *testRootDir
		case "diff":
			config.Diff = *diff
		case "dfilter":
			config.DiffFilter = *diffFilter
		case "testdir":
			config.TestDir = *testdir
		case "noskip":
			config.Skips = !*noskip
		case "timings":
			config.Timings = *timings
		case "timeout":
			config.Timeout = *timeout
		case "i":
			config.Interactive = *interactive
		case "stats":
			config.ShowStats = *showStats
		case "v":
			config.Verbose = *verbose
		case "rebuild":
			config.Rebuild = *rebuild
		case "build-mode":
			mode, err := build.ParseBuildMode(*buildMode)
			if err != nil {
				log.Fatalf("Invalid build mode: %v", err)
			}
			config.BuildMode = mode
		case "compiler":
			compiler, err := build.ParseCompiler(*buildCompiler)
			if err != nil {
				log.Fatalf("Invalid compiler: %v", err)
			}
			config.BuildCompiler = compiler
		}
	})

	args := flag.Args()
	if len(args) > 0 {
		config.FilterPath = args[0]
	}

	if config.Selection != "full" && config.Selection != "quick" {
		log.Fatal("selection must be full or quick")
	}
	if math.IsNaN(config.Coverage) || math.IsInf(config.Coverage, 0) || config.Coverage <= 0 || config.Coverage > 100 {
		log.Fatal("coverage must be in (0,100]")
	}
	if config.Workers < 0 || config.RiskyWorkers < 0 || config.RiskyCPUs < 0 || config.CPUCount < 0 {
		log.Fatal("worker and CPU counts must be nonnegative")
	}
	if config.Timeout <= 0 {
		log.Fatal("timeout must be positive")
	}
	if config.BenchSamples < 8 || config.BenchRepeats < 1 {
		log.Fatal("benchmark needs at least 8 samples and 1 repeat")
	}
	if d, err := time.ParseDuration(config.ProbeTimeout); err != nil || d <= 0 {
		log.Fatal("probe-timeout must be a positive duration")
	}
	if config.CI && config.Selection == "quick" {
		log.Fatal("quick results cannot replace full CI history; run quick without --ci")
	}
	if config.CalibrationPath == "" {
		config.CalibrationPath = "runner-calibration.json"
		if config.RepoPath != "" {
			config.CalibrationPath = filepath.Join(config.RepoPath, "runner-calibration.json")
		}
	}
	if config.HistoryPath == "" {
		config.HistoryPath = "results.json"
		if config.RepoPath != "" {
			config.HistoryPath = filepath.Join(config.RepoPath, "results.json")
		}
	}
	if config.CI && config.ReportPath == "" {
		config.ReportPath = filepath.Join(config.RepoPath, "runner-timings.json")
	}
	return config
}

func NormalizeFilterPath(filterPath string, testRoot string) string {
	if filterPath == "" {
		return ""
	}

	prefixes := []string{
		"../../test262/test/",
		"../test262/test/",
		"test262/test/",
		"test/",
	}

	result := filterPath
	for _, prefix := range prefixes {
		if strings.HasPrefix(result, prefix) {
			result = strings.TrimPrefix(result, prefix)
			break
		}
	}

	if filepath.IsAbs(result) {
		if rel, err := filepath.Rel(testRoot, result); err == nil {
			result = rel
		}
	}

	return result
}

func loadProfile(filename string, profileName string, config *Config) error {
	contents, err := os.ReadFile(filename)
	if err != nil {
		return fmt.Errorf("failed to read profiles file: %w", err)
	}

	var profileConfig ProfileConfig
	if err := json.Unmarshal(contents, &profileConfig); err != nil {
		return fmt.Errorf("failed to parse profiles file: %w", err)
	}

	profile, exists := profileConfig.Profiles[profileName]
	if !exists {
		return fmt.Errorf("profile '%s' not found in profiles file", profileName)
	}

	// Apply profile settings to config
	if profile.RiskyWorkers != nil {
		config.RiskyWorkers = *profile.RiskyWorkers
	}
	if profile.RiskyCPUs != nil {
		config.RiskyCPUs = *profile.RiskyCPUs
	}
	if profile.CPUCount != nil {
		config.CPUCount = *profile.CPUCount
	}
	if profile.Selection != nil {
		config.Selection = *profile.Selection
	}
	if profile.Coverage != nil {
		config.Coverage = *profile.Coverage
	}
	if profile.CalibrationPath != nil {
		config.CalibrationPath = *profile.CalibrationPath
	}
	if profile.CalibrationKey != nil {
		config.CalibrationKey = *profile.CalibrationKey
	}
	if profile.HistoryPath != nil {
		config.HistoryPath = *profile.HistoryPath
	}
	if profile.Engine != nil {
		config.Engine = *profile.Engine
	}
	if profile.ReportPath != nil {
		config.ReportPath = *profile.ReportPath
	}
	if profile.Bench != nil {
		config.Bench = *profile.Bench
	}
	if profile.BenchSamples != nil {
		config.BenchSamples = *profile.BenchSamples
	}
	if profile.BenchRepeats != nil {
		config.BenchRepeats = *profile.BenchRepeats
	}
	if profile.BenchOutput != nil {
		config.BenchOutput = *profile.BenchOutput
	}
	if profile.ProbeTimeout != nil {
		config.ProbeTimeout = *profile.ProbeTimeout
	}
	if profile.Output != nil {
		config.Output = *profile.Output
	}

	if profile.CI != nil {
		config.CI = *profile.CI
	}
	if profile.RepoPath != nil {
		config.RepoPath = *profile.RepoPath
	}
	if profile.HistoryOnly != nil {
		config.HistoryOnly = *profile.HistoryOnly
	}
	if profile.Workers != nil {
		config.Workers = *profile.Workers
	}
	if profile.TestRootDir != nil {
		config.TestRootDir = *profile.TestRootDir
	}
	if profile.Diff != nil {
		config.Diff = *profile.Diff
	}
	if profile.DiffFilter != nil {
		config.DiffFilter = *profile.DiffFilter
	}
	if profile.TestDir != nil {
		config.TestDir = *profile.TestDir
	}
	if profile.NoSkip != nil {
		config.Skips = !*profile.NoSkip
	}
	if profile.Timings != nil {
		config.Timings = *profile.Timings
	}
	if profile.Timeout != nil {
		duration, err := time.ParseDuration(*profile.Timeout)
		if err != nil {
			return fmt.Errorf("invalid timeout in profile: %w", err)
		}
		config.Timeout = duration
	}
	if profile.Interactive != nil {
		config.Interactive = *profile.Interactive
	}
	if profile.ShowStats != nil {
		config.ShowStats = *profile.ShowStats
	}
	if profile.Verbose != nil {
		config.Verbose = *profile.Verbose
	}

	if profile.Rebuild != nil {
		config.Rebuild = *profile.Rebuild
	}
	if profile.BuildMode != nil {
		mode, err := build.ParseBuildMode(*profile.BuildMode)
		if err != nil {
			return fmt.Errorf("invalid build mode in profile: %w", err)
		}
		config.BuildMode = mode
	}
	if profile.BuildCompiler != nil {
		compiler, err := build.ParseCompiler(*profile.BuildCompiler)
		if err != nil {
			return fmt.Errorf("invalid compiler in profile: %w", err)
		}
		config.BuildCompiler = compiler
	}

	return nil
}
