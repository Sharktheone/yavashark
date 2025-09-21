package main

import (
	"flag"
	"log"
	"os"

	"github.com/BurntSushi/toml"
)

type Config struct {
	CI          bool   `toml:"ci"`
	RepoPath    string `toml:"repo_path"`
	HistoryOnly bool   `toml:"history_only"`
	Workers     int    `toml:"workers"`
	TestRootDir string `toml:"test_root_dir"`
	Diff        bool   `toml:"diff"`
	DiffFilter  string `toml:"diff_filter"`
	TestDir     string `toml:"test_dir"`
	Skips       bool   `toml:"skips"`
	Timings     bool   `toml:"timings"`
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
	}
}

func LoadConfig() *Config {
	config := NewConfig()

	configFile := flag.String("config", "config.toml", "Path to TOML config file")
	ciEnabled := flag.Bool("ci", config.CI, "Enable CI mode to commit results")
	repoPath := flag.String("repo", config.RepoPath, "Path to external repository for CI results")
	historyOnly := flag.Bool("history-only", config.HistoryOnly, "Only generate the history file (skip git commit)")
	workers := flag.Int("workers", config.Workers, "Number of workers")
	testRootDir := flag.String("test_root", config.TestRootDir, "Path to test root directory")
	diff := flag.Bool("diff", config.Diff, "Diff to use for CI results")
	diffFilter := flag.String("dfilter", config.DiffFilter, "Diff filter to use for CI results")
	testdir := flag.String("testdir", config.TestDir, "Path in the test directory")
	noskip := flag.Bool("noskip", false, "Path in the test directory")
	timings := flag.Bool("timings", false, "Attempt to parse timings from test output (if enabled)")

	flag.Parse()

	if *configFile != "" {
		if err := loadConfigFile(*configFile, config); err != nil {
			if !os.IsNotExist(err) {
				log.Fatalf("Failed to load config file %s: %v", *configFile, err)
			}
		}
	}

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
		}
	})

	return config
}

func loadConfigFile(filename string, config *Config) error {
	_, err := toml.DecodeFile(filename, config)
	return err
}
