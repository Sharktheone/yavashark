package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestProfileSchedulingOptions(t *testing.T) {
	p := filepath.Join(t.TempDir(), "profiles.json")
	if err := os.WriteFile(p, []byte(`{"profiles":{"bench":{"bench":true,"cpus":4,"risky_cpus":1,"risky_workers":12,"workers":48,"selection":"quick","coverage":25,"calibration":"measurements.json","bench_samples":256,"bench_repeats":3,"probe_timeout":"1s"}}}`), 0600); err != nil {
		t.Fatal(err)
	}
	c := NewConfig()
	if err := loadProfile(p, "bench", c); err != nil {
		t.Fatal(err)
	}
	if !c.Bench || c.CPUCount != 4 || c.RiskyCPUs != 1 || c.RiskyWorkers != 12 || c.Workers != 48 || c.Coverage != 25 || c.Selection != "quick" || c.CalibrationPath != "measurements.json" || c.BenchSamples != 256 || c.BenchRepeats != 3 || c.ProbeTimeout != "1s" {
		t.Fatalf("%+v", c)
	}
}
