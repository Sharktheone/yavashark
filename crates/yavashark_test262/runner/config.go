package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"
	"time"
	"yavashark_test262_runner/build"
)

const (
	DEFAULT_PROFILE_FILE = "profiles.json"
)

type Config struct {
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
		CI:          false,
		RepoPath:    "",
		HistoryOnly: false,
		Workers:     DEFAULT_WORKERS,
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
	workers := flag.Int("workers", config.Workers, "Number of workers")
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

	flag.Parse()

	// Load profile if specified
	if *profile != "" {
		if err := loadProfile(*profileFile, *profile, config); err != nil {
			log.Printf("Warning: Failed to load profile '%s': %v", *profile, err)
		}
	}

	// Override with command-line flags (flags take precedence over profile)
	flag.Visit(func(f *flag.Flag) {
		switch f.Name {
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
