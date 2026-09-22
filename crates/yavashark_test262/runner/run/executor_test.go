package run

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
	"time"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/cpus"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/scheduler"
	"yavashark_test262_runner/status"
)

func TestRiskyHangDoesNotBlockNormalQueue(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("shell fixture")
	}
	dir := t.TempDir()
	engine := filepath.Join(dir, "engine")
	// Busy-loop in the child itself, avoiding shell grandchildren surviving timeout.
	if err := os.WriteFile(engine, []byte("#!/bin/sh\nif [ \"$1\" = hang ]; then while :; do :; done; fi\necho PASS\n"), 0755); err != nil {
		t.Fatal(err)
	}
	ids, err := cpus.Budget(0)
	if err != nil {
		t.Fatal(err)
	}
	p := scheduler.Plan{Fast: []scheduler.Job{{Path: "fast"}, {Path: "second"}}, Risky: []scheduler.Job{{Path: "hang"}}}
	var got []results.Result
	stats := Execute(p, Execution{CPUs: ids, Engine: engine, Timeout: 300 * time.Millisecond, Settings: calibration.Settings{RiskyCPUs: 1, FastWorkers: 1, RiskyWorkers: 1}}, func(r results.Result) { got = append(got, r) })
	if len(got) != 3 || got[0].Status != status.PASS || got[2].Status != status.TIMEOUT {
		t.Fatalf("unexpected order/results: %+v", got)
	}
	if stats.Duration > 2*time.Second {
		t.Fatal("queue failed to progress")
	}
}
func TestDiscoveryDeduplicatesAndOmitsFixtures(t *testing.T) {
	d := t.TempDir()
	for _, p := range []string{"a.js", "x_FIXTURE.js", "README.md"} {
		if err := os.WriteFile(filepath.Join(d, p), nil, 0600); err != nil {
			t.Fatal(err)
		}
	}
	files, _, errs := Discover([]string{d, filepath.Join(d, "a.js")}, false)
	if len(files) != 1 || len(errs) != 0 {
		t.Fatalf("%v %v", files, errs)
	}
}

func TestExtensionlessTest262FileIsDiscovered(t *testing.T) {
	d := t.TempDir()
	p := filepath.Join(d, "upstream-test-without-extension")
	if err := os.WriteFile(p, []byte("// Copyright\n/*---\ndescription: test\n---*/\n"), 0600); err != nil {
		t.Fatal(err)
	}
	files, _, errs := Discover([]string{d}, false)
	if len(files) != 1 || len(errs) != 0 {
		t.Fatalf("%v %v", files, errs)
	}
}

func TestCompletedPoolLendsCPUsToPendingJobs(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("shell fixture")
	}
	d := t.TempDir()
	engine := filepath.Join(d, "engine")
	if err := os.WriteFile(engine, []byte("#!/bin/sh\nif [ \"$1\" != risk ]; then sleep 0.04; fi\necho PASS\n"), 0755); err != nil {
		t.Fatal(err)
	}
	ids, err := cpus.Budget(0)
	if err != nil {
		t.Fatal(err)
	}
	p := scheduler.Plan{Fast: []scheduler.Job{{Path: "a"}, {Path: "b"}, {Path: "c"}}, Risky: []scheduler.Job{{Path: "risk"}}}
	s := Execute(p, Execution{CPUs: ids, Engine: engine, Timeout: time.Second, Settings: calibration.Settings{RiskyCPUs: 1, FastWorkers: 1, RiskyWorkers: 1}}, func(r results.Result) {
		if r.Status != status.PASS {
			t.Errorf("%+v", r)
		}
	})
	if s.NormalBorrowed < 1 {
		t.Fatalf("idle risky CPUs were never lent: %+v", s)
	}
}

func TestQuickRunPersistsAveragesAndLeavesFullResultsAlone(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("shell fixture")
	}
	d := t.TempDir()
	root := filepath.Join(d, "test262", "test", "language")
	if err := os.MkdirAll(root, 0755); err != nil {
		t.Fatal(err)
	}
	for _, name := range []string{"a.js", "b.js", "c.js", "d.js", "hang.js"} {
		code := "echo PASS\n"
		if name == "hang.js" {
			code = "while :; do :; done\n"
		}
		if err := os.WriteFile(filepath.Join(root, name), []byte(code), 0600); err != nil {
			t.Fatal(err)
		}
	}
	history := filepath.Join(d, "results.json")
	if err := calibration.Save(history, []results.CIResult{{Path: "language/hang.js", Status: status.CI_TIMEOUT}}); err != nil {
		t.Fatal(err)
	}
	before, err := os.ReadFile(history)
	if err != nil {
		t.Fatal(err)
	}
	store := filepath.Join(d, "calibration.json")
	report := filepath.Join(d, "timings.json")
	cfg := RunConfig{Selection: "quick", Coverage: 50, Workers: 2, RiskyWorkers: 1, Engine: "/bin/sh", Timeout: time.Second, HistoryPath: history, CalibrationPath: store, CalibrationKey: "fixture", ReportPath: report}
	for i := 0; i < 2; i++ {
		tr, _, _, err := runPaths([]string{root}, cfg)
		if err != nil {
			t.Fatal(err)
		}
		if tr.Total != 2 || tr.Passed != 2 {
			t.Fatalf("%+v", tr)
		}
	}
	after, err := os.ReadFile(history)
	if err != nil {
		t.Fatal(err)
	}
	if string(before) != string(after) {
		t.Fatal("quick run replaced full results")
	}
	f, err := calibration.Load(store)
	if err != nil {
		t.Fatal(err)
	}
	e := f.Environments["fixture"]
	if e.Runs != 2 || len(e.Tests) != 2 {
		t.Fatalf("%+v", e)
	}
	for _, m := range e.Tests {
		if m.Samples != 2 {
			t.Fatalf("%+v", m)
		}
	}
	if _, err = os.Stat(report); err != nil {
		t.Fatal(err)
	}
}

func TestExplicitNormalWorkerLimitAppliesAcrossPhases(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("shell fixture")
	}
	d := t.TempDir()
	engine := filepath.Join(d, "engine")
	t.Setenv("RUNNER_TEST_LOCK", filepath.Join(d, "lock"))
	code := "#!/bin/sh\nif ! mkdir \"$RUNNER_TEST_LOCK\" 2>/dev/null; then echo FAIL; exit; fi\nsleep 0.02\nrmdir \"$RUNNER_TEST_LOCK\"\necho PASS\n"
	if err := os.WriteFile(engine, []byte(code), 0755); err != nil {
		t.Fatal(err)
	}
	ids, err := cpus.Budget(0)
	if err != nil {
		t.Fatal(err)
	}
	jobs := []scheduler.Job{{Path: "a"}, {Path: "b"}}
	p := scheduler.Plan{Fast: jobs, Medium: jobs, Slow: jobs}
	count := 0
	Execute(p, Execution{NormalLimit: 1, CPUs: ids, Engine: engine, Timeout: time.Second, Settings: calibration.Settings{FastWorkers: 2, MediumWorkers: 2, SlowWorkers: 2}}, func(r results.Result) {
		count++
		if r.Status != status.PASS {
			t.Errorf("normal concurrency cap exceeded: %+v", r)
		}
	})
	if count != 6 {
		t.Fatalf("got %d results", count)
	}
}

func TestPreparedInputsAreReusedAndMatchPaths(t *testing.T) {
	d := t.TempDir()
	root := filepath.Join(d, "tests")
	if err := os.Mkdir(root, 0755); err != nil {
		t.Fatal(err)
	}
	cfg := RunConfig{CalibrationPath: filepath.Join(d, "cal.json"), HistoryPath: filepath.Join(d, "history.json")}
	prepared, err := Prepare([]string{root}, cfg)
	if err != nil {
		t.Fatal(err)
	}
	cfg.Prepared = prepared
	// The already loaded inputs remain usable after the files change/disappear.
	if err := os.Remove(root); err != nil {
		t.Fatal(err)
	}
	reused, err := preparedInputs([]string{root}, cfg)
	if err != nil || reused != prepared {
		t.Fatalf("%v %v", reused, err)
	}
	if _, err = preparedInputs([]string{"another-root"}, cfg); err == nil {
		t.Fatal("accepted mismatched discovery roots")
	}
}
