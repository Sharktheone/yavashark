package run

import (
	"crypto/sha256"
	"fmt"
	"hash/fnv"
	"log"
	"math"
	"sort"
	"time"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/cpus"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/scheduler"
	"yavashark_test262_runner/status"
)

type BenchConfig struct {
	RunConfig
	Samples      int
	Repeats      int
	ProbeTimeout time.Duration
	Output       string
}
type Trial struct {
	CPUSeconds         float64       `json:"cpu_seconds"`
	UnexpectedFailures int           `json:"unexpected_failures"`
	HistoryRuns        uint64        `json:"history_runs"`
	Tier               string        `json:"tier"`
	CPUs               int           `json:"cpus"`
	Workers            int           `json:"workers"`
	Repeats            int           `json:"repeats"`
	Tests              int           `json:"tests"`
	Completed          int           `json:"completed"`
	Timeouts           int           `json:"timeouts"`
	Duration           time.Duration `json:"duration_ns"`
	MemoryKB           uint64        `json:"max_child_memory_kb"`
	Score              float64       `json:"projected_seconds"`
}
type Candidate struct {
	Settings           calibration.Settings `json:"settings"`
	Projected          float64              `json:"projected_seconds"`
	Validation         time.Duration        `json:"validation_ns"`
	ValidationTimeouts int                  `json:"validation_timeouts"`
	ValidationFailures int                  `json:"validation_unexpected_failures"`
}
type BenchReport struct {
	Version       int                  `json:"version"`
	CPUs          []int                `json:"cpus"`
	Isolation     string               `json:"isolation"`
	Started       string               `json:"started"`
	Duration      time.Duration        `json:"duration_ns"`
	Trials        []Trial              `json:"trials"`
	Candidates    []Candidate          `json:"candidates"`
	Recommended   calibration.Settings `json:"recommended"`
	ProbeTimeout  time.Duration        `json:"probe_timeout_ns"`
	TargetTimeout time.Duration        `json:"target_timeout_ns"`
	Note          string               `json:"note"`
}

func sample(jobs []scheduler.Job, n int) []scheduler.Job {
	// Stable across timing updates so repeat calibrations can average the same
	// workload. Hash ordering spreads samples across directories without cost bias.
	sorted := append([]scheduler.Job(nil), jobs...)
	hash := func(path string) uint64 { h := fnv.New64a(); h.Write([]byte(calibration.Key(path))); return h.Sum64() }
	sort.Slice(sorted, func(i, j int) bool {
		a, b := hash(sorted[i].Path), hash(sorted[j].Path)
		if a == b {
			return sorted[i].Path < sorted[j].Path
		}
		return a < b
	})
	return sorted[:min(n, len(sorted))]
}

func unique(values ...int) []int {
	seen := map[int]bool{}
	var out []int
	for _, v := range values {
		v = max(1, v)
		if !seen[v] {
			seen[v] = true
			out = append(out, v)
		}
	}
	sort.Ints(out)
	return out
}

func Benchmark(paths []string, cfg BenchConfig) error {
	began := time.Now()
	inputs, err := preparedInputs(paths, cfg.RunConfig)
	if err != nil {
		return err
	}
	tests := inputs.tests
	if len(tests) == 0 {
		return fmt.Errorf("no benchmark tests")
	}
	cfg.Prepared = inputs
	store, env, base, err := resolve(cfg.RunConfig)
	if err != nil {
		return err
	}
	history := inputs.history

	plan := scheduler.Make(tests, env, history, "full", 100)
	report := BenchReport{Version: 1, CPUs: base.CPUs, Isolation: cpus.Isolation(), Started: time.Now().UTC().Format(time.RFC3339), ProbeTimeout: cfg.ProbeTimeout, TargetTimeout: cfg.Timeout, Note: "Calibration trials deliberately repeat samples; ordinary runs never retry. Probe timeouts are censored and do not update test classifications. Projected times are estimates, not full-suite timings."}
	n := max(8, cfg.Samples)
	repeats := max(1, cfg.Repeats)
	samples := [][]scheduler.Job{sample(plan.Fast, n), sample(plan.Medium, min(n, 128)), sample(plan.Slow, min(n, 24)), sample(plan.Risky, min(n, 24))}
	populations := []int{len(plan.Fast), len(plan.Medium), len(plan.Slow), len(plan.Risky)}
	names := []string{"fast", "medium", "slow", "risky"}
	cache := map[string]Trial{}
	reference := map[string]status.Status{}
	if env.Benchmarks == nil {
		env.Benchmarks = map[string]*calibration.BenchAverage{}
	}
	trial := func(tier, workers int, ids []int) Trial {
		key := fmt.Sprintf("%d/%d/%v", tier, workers, ids)
		if t, ok := cache[key]; ok {
			return t
		}
		t := Trial{Tier: names[tier], CPUs: len(ids), Workers: workers, Repeats: repeats}
		for rep := 0; rep < repeats; rep++ {
			p := scheduler.Plan{Fast: samples[tier]}
			opts := base
			opts.NormalLimit = 0
			opts.CPUs = ids
			opts.LowPriority = tier == 3
			opts.Settings.FastWorkers = workers
			opts.Timeout = cfg.ProbeTimeout
			st := Execute(p, opts, func(r results.Result) {
				t.Tests++
				t.CPUSeconds += r.CPUTime.Seconds()
				previous, known := reference[r.Path]
				bad := r.Status == status.TIMEOUT || r.Status == status.CRASH || r.Status == status.RUNNER_ERROR
				if r.Status == status.RUNNER_ERROR || (known && bad && previous != r.Status) {
					t.UnexpectedFailures++
				}
				if !known && !bad {
					reference[r.Path] = r.Status
				}
				if r.Status == status.TIMEOUT {
					t.Timeouts++
				} else if r.Status != status.RUNNER_ERROR {
					t.Completed++
				}
				t.MemoryKB = max(t.MemoryKB, r.MemoryKB)
			})
			t.Duration += st.Duration
		}
		if t.Tests > 0 {
			signature := fmt.Sprintf("%s/%s", key, cfg.ProbeTimeout)
			for _, j := range samples[tier] {
				signature += "/" + calibration.Key(j.Path)
			}
			historyKey := fmt.Sprintf("%x", sha256.Sum256([]byte(signature)))
			avg := env.Benchmarks[historyKey]
			if avg == nil {
				avg = &calibration.BenchAverage{}
				env.Benchmarks[historyKey] = avg
			}
			avg.Runs++
			avg.Seconds += (t.Duration.Seconds()/float64(repeats) - avg.Seconds) / float64(avg.Runs)
			t.HistoryRuns = avg.Runs
			t.Score = avg.Seconds * float64(populations[tier]) / float64(len(samples[tier]))
		}
		cache[key] = t
		report.Trials = append(report.Trials, t)
		log.Printf("Bench %s: CPUs=%d workers=%d completed=%d/%d duration=%s", t.Tier, t.CPUs, t.Workers, t.Completed, t.Tests, t.Duration)
		return t
	}
	splits := unique(1, len(base.CPUs)/4, len(base.CPUs)/2)
	if len(plan.Risky) == 0 || len(base.CPUs) < 2 {
		splits = []int{0}
	}
	seenMasks := map[string]bool{}
	for _, split := range splits {
		normal, risk := cpus.Split(base.CPUs, split)
		if len(risk) == 0 {
			risk = normal
		}
		if len(plan.Risky) == 0 {
			normal = base.CPUs
		}
		maskKey := fmt.Sprint(normal)
		if seenMasks[maskKey] {
			continue
		}
		seenMasks[maskKey] = true
		settings := DefaultSettings(len(base.CPUs))
		settings.RiskyCPUs = len(risk)
		if len(plan.Risky) == 0 {
			settings.RiskyCPUs = DefaultSettings(len(base.CPUs)).RiskyCPUs
		}
		var projected [4]float64
		for tier := range names {
			ids := normal
			if tier == 3 {
				ids = risk
			}
			c := len(ids)
			workers := unique(max(1, c/2), c, c*2, c*4, c*16, c*64)
			if tier == 1 {
				workers = unique(c, c*2, c*4, c*8)
			}
			if tier == 2 {
				workers = unique(max(1, c/2), c, c*2)
			}
			if len(samples[tier]) == 0 {
				continue
			}
			// Samples must actually exercise the tested concurrency; no fictional 1024-worker result from 24 jobs.
			var ts []Trial
			seen := map[int]bool{}
			for _, w := range workers {
				w = min(w, len(samples[tier]))
				if seen[w] {
					continue
				}
				seen[w] = true
				ts = append(ts, trial(tier, w, ids))
			}
			minFailures := ts[0].UnexpectedFailures
			for _, t := range ts {
				minFailures = min(minFailures, t.UnexpectedFailures)
			}
			maxCompleted := 0
			for _, t := range ts {
				if t.UnexpectedFailures == minFailures {
					maxCompleted = max(maxCompleted, t.Completed)
				}
			}
			best := math.Inf(1)
			for _, t := range ts {
				if t.UnexpectedFailures == minFailures && t.Completed == maxCompleted {
					best = min(best, t.Score)
				}
			}
			chosen := ts[0]
			for _, t := range ts {
				if t.UnexpectedFailures == minFailures && t.Completed == maxCompleted && t.Score <= best*1.05 {
					chosen = t
					break
				}
			}
			switch tier {
			case 0:
				settings.FastWorkers = chosen.Workers
			case 1:
				settings.MediumWorkers = chosen.Workers
			case 2:
				settings.SlowWorkers = chosen.Workers
			case 3:
				settings.RiskyWorkers = chosen.Workers
			}
			projected[tier] = chosen.Score
		}
		// A bounded sample cannot directly tune hundreds of known hangs. Launch
		// their wall-clock timers in parallel, capped to a documented process budget.
		// They remain confined to the risky CPUs and never borrow normal workers.
		hangs := 0
		for _, j := range plan.Risky {
			if j.TimedOut {
				hangs++
			}
		}
		// Normal phases share CPUs sequentially; the risky pool overlaps them.
		normalProjection := projected[0] + projected[1] + projected[2]
		targetTimeout := cfg.Timeout
		if targetTimeout <= 0 {
			targetTimeout = 30 * time.Second
		}
		// Fit known timeout waves beneath normal work where possible. This avoids
		// a fixed small worker cap turning 700 hangs into minutes of serial batches.
		waves := max(1, int(normalProjection/targetTimeout.Seconds()))
		needed := (hangs + waves - 1) / waves
		measuredRiskyWorkers := settings.RiskyWorkers
		settings.RiskyWorkers = max(settings.RiskyWorkers, min(needed, max(128, len(base.CPUs)*16)))
		projected[3] *= float64(measuredRiskyWorkers) / float64(settings.RiskyWorkers)
		projected[3] = max(projected[3], math.Ceil(float64(hangs)/float64(settings.RiskyWorkers))*targetTimeout.Seconds())

		report.Candidates = append(report.Candidates, Candidate{Settings: settings, Projected: max(normalProjection, projected[3])})
	}
	// Validate each candidate with both pools active. Use identical bounded samples.
	best := -1
	for i := range report.Candidates {
		c := &report.Candidates[i]
		// Preserve the population mix; equal tier-sized samples overrepresent
		// expensive jobs and incorrectly reward excessive risky CPU reservations.
		scale := float64(n) / float64(max(1, len(plan.Fast)))
		validation := func(jobs []scheduler.Job) []scheduler.Job {
			return sample(jobs, max(1, int(math.Ceil(float64(len(jobs))*scale))))
		}
		p := scheduler.Plan{Fast: samples[0], Medium: validation(plan.Medium), Slow: validation(plan.Slow), Risky: validation(plan.Risky)}
		opts := base
		opts.NormalLimit = 0
		opts.Settings = c.Settings
		opts.Timeout = cfg.Timeout
		st := Execute(p, opts, func(r results.Result) {
			if r.Status == status.TIMEOUT {
				c.ValidationTimeouts++
			}
			previous, known := reference[r.Path]
			bad := r.Status == status.TIMEOUT || r.Status == status.CRASH || r.Status == status.RUNNER_ERROR
			if r.Status == status.RUNNER_ERROR || (known && bad && previous != r.Status) {
				c.ValidationFailures++
			}
		})
		c.Validation = st.Duration
		if best < 0 || c.ValidationFailures < report.Candidates[best].ValidationFailures || (c.ValidationFailures == report.Candidates[best].ValidationFailures && c.Projected < report.Candidates[best].Projected) {
			best = i
		}
	}
	report.Recommended = report.Candidates[best].Settings
	env.Settings = &report.Recommended
	env.BenchRuns++
	env.Updated = time.Now().UTC().Format(time.RFC3339)
	report.Duration = time.Since(began)
	file := cfg.CalibrationPath
	if file == "" {
		file = "runner-calibration.json"
	}
	if err = calibration.Save(file, store); err != nil {
		return err
	}
	output := cfg.Output
	if output == "" {
		output = "runner-bench.json"
	}
	if err = calibration.Save(output, report); err != nil {
		return err
	}
	log.Printf("Benchmark finished in %s; recommended %+v; saved %s", report.Duration, report.Recommended, output)
	return nil
}
