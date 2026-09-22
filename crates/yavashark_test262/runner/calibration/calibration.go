// Package calibration stores measured costs, never estimates from source size.
package calibration

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
)

type Tier string

const (
	Fast    Tier = "fast"
	Medium  Tier = "medium"
	Slow    Tier = "slow"
	Risky   Tier = "risky"
	Unknown Tier = "unknown"
)

type Measurement struct {
	Samples    uint64        `json:"samples"`
	CPU        float64       `json:"cpu_ns"`
	Wall       float64       `json:"wall_ns"`
	Memory     float64       `json:"memory_kb"`
	LastStatus status.Status `json:"last_status"`
	Timeouts   uint64        `json:"timeouts"`
}
type Settings struct {
	RiskyCPUs     int `json:"risky_cpus"`
	FastWorkers   int `json:"fast_workers"`
	MediumWorkers int `json:"medium_workers"`
	SlowWorkers   int `json:"slow_workers"`
	RiskyWorkers  int `json:"risky_workers"`
}
type BenchAverage struct {
	Runs    uint64  `json:"runs"`
	Seconds float64 `json:"mean_seconds"`
}
type Environment struct {
	Benchmarks map[string]*BenchAverage `json:"benchmarks,omitempty"`
	OS         string                   `json:"os"`
	CPUs       int                      `json:"cpus"`
	Runs       uint64                   `json:"runs"`
	Tests      map[string]*Measurement  `json:"tests"`
	Settings   *Settings                `json:"settings,omitempty"`
	BenchRuns  uint64                   `json:"bench_runs,omitempty"`
	Updated    string                   `json:"updated"`
}
type File struct {
	Version      int                     `json:"version"`
	Environments map[string]*Environment `json:"environments"`
}

func Load(path string) (*File, error) {
	f := &File{Version: 1, Environments: map[string]*Environment{}}
	b, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return f, nil
	}
	if err != nil {
		return nil, err
	}
	if err = json.Unmarshal(b, f); err != nil {
		return nil, err
	}
	if f.Version != 1 {
		return nil, fmt.Errorf("unsupported calibration version %d", f.Version)
	}
	if f.Environments == nil {
		f.Environments = map[string]*Environment{}
	}
	return f, nil
}
func (f *File) Environment(key string, cpus int) *Environment {
	e := f.Environments[key]
	if e == nil {
		e = &Environment{OS: runtime.GOOS, CPUs: cpus, Tests: map[string]*Measurement{}}
		f.Environments[key] = e
	}
	if e.Tests == nil {
		e.Tests = map[string]*Measurement{}
	}
	return e
}
func Save(path string, value any) error {
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}
	b, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return err
	}
	f, err := os.CreateTemp(filepath.Dir(path), ".runner-*.json")
	if err != nil {
		return err
	}
	name := f.Name()
	defer os.Remove(name)
	if _, err = f.Write(append(b, '\n')); err != nil {
		f.Close()
		return err
	}
	if err = f.Close(); err != nil {
		return err
	}
	return os.Rename(name, path)
}

// Key is independent of checkout location, including CI's compact result paths.
func Key(path string) string {
	s := filepath.ToSlash(filepath.Clean(path))
	if i := strings.LastIndex(s, "test262/test/"); i >= 0 {
		return s[i+len("test262/test/"):]
	}
	return strings.TrimPrefix(s, "test/")
}
func CostTier(cpu float64) Tier {
	switch {
	case cpu > float64(5*time.Second):
		return Risky
	case cpu > float64(100*time.Millisecond):
		return Slow
	case cpu > float64(10*time.Millisecond):
		return Medium
	default:
		return Fast
	}
}
func (e *Environment) Observe(rs []results.Result) {
	for _, r := range rs {
		if r.Status == status.SKIP || r.Status == status.RUNNER_ERROR {
			continue
		}
		k := Key(r.Path)
		m := e.Tests[k]
		if m == nil {
			m = &Measurement{}
			e.Tests[k] = m
		}
		m.LastStatus = r.Status
		if r.Status == status.TIMEOUT {
			m.Timeouts++
			continue
		} // censored observations aren't completed durations
		m.Samples++
		n := float64(m.Samples)
		m.CPU += (float64(r.CPUTime) - m.CPU) / n
		m.Wall += (float64(r.Duration) - m.Wall) / n
		m.Memory += (float64(r.MemoryKB) - m.Memory) / n
	}
	e.Runs++
	e.Updated = time.Now().UTC().Format(time.RFC3339)
}

// History reads either local detailed results or CI's compact status-only file.
// No fabricated duration is assigned to a timeout, crash, or missing timing.
func History(path string) (map[string]results.Result, error) {
	out := map[string]results.Result{}
	if path == "" {
		return out, nil
	}
	b, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return out, nil
	}
	if err != nil {
		return nil, err
	}
	// Decode only scheduling fields, once. Parsing each row into RawMessage,
	// a map and then a full Result was allocating discarded failure messages.
	var rows []struct {
		Status        status.Status   `json:"status"`
		Path          string          `json:"path"`
		CPUTime       time.Duration   `json:"cpu_time"`
		Duration      time.Duration   `json:"duration"`
		CompactStatus status.CIStatus `json:"s"`
		CompactPath   string          `json:"p"`
	}
	if err = json.Unmarshal(b, &rows); err != nil {
		return nil, err
	}
	for _, row := range rows {
		r := results.Result{Status: row.Status, Path: row.Path, CPUTime: row.CPUTime, Duration: row.Duration}
		if row.CompactPath != "" {
			r.Path = row.CompactPath
			r.Status = row.CompactStatus.ToStatus()
		}
		if r.Path != "" {
			out[Key(r.Path)] = r
		}
	}

	return out, nil
}
