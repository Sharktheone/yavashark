package run

import (
	"fmt"
	"log"
	"os/exec"
	"runtime"
	"sync"
	"sync/atomic"
	"time"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/cpus"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/scheduler"
	"yavashark_test262_runner/test"
)

// Options are also used by the benchmark so trials exercise the real scheduler.
type Execution struct {
	NormalLimit int  // explicit --workers cap for normal phases
	LowPriority bool // benchmark probes for the risky pool
	Settings    calibration.Settings
	CPUs        []int
	Engine      string
	Timeout     time.Duration
	Timings     bool
}

type Phase struct {
	Name     string        `json:"name"`
	Tests    int           `json:"tests"`
	Workers  int           `json:"workers"`
	Duration time.Duration `json:"duration_ns"`
}
type ExecutionStats struct {
	NormalBorrowed int64         `json:"normal_jobs_using_idle_risky_cpus"`
	RiskyBorrowed  int64         `json:"risky_jobs_using_idle_normal_cpus"`
	Duration       time.Duration `json:"duration_ns"`
	Phases         []Phase       `json:"phases"`
	NormalCPUs     []int         `json:"normal_cpus"`
	RiskyCPUs      []int         `json:"risky_cpus"`
}

func DefaultSettings(n int) calibration.Settings {
	risky := max(1, n/4)
	normal := max(1, n-risky)
	// Conservative bootstrap counts for a first run without calibration. Fast
	// children need launch overlap, but hundreds of processes hurt throughput.
	slow := normal
	if normal <= 8 {
		slow *= 2
	}
	return calibration.Settings{RiskyCPUs: risky, FastWorkers: max(16, normal), MediumWorkers: normal * 2, SlowWorkers: slow, RiskyWorkers: risky * 4}
}

// Execute has independent pool lifetimes: an infinite loop cannot occupy a
// normal worker. Each test is launched once, with its timer starting at launch.
func Execute(plan scheduler.Plan, opts Execution, emit func(results.Result)) ExecutionStats {
	start := time.Now()
	normal, risk := cpus.Split(opts.CPUs, opts.Settings.RiskyCPUs)
	if len(plan.Risky) == 0 {
		normal = opts.CPUs
		risk = nil
	}
	if len(risk) == 0 && len(plan.Risky) > 0 {
		risk = normal
	}
	stats := ExecutionStats{NormalCPUs: normal, RiskyCPUs: risk}
	var mu sync.Mutex
	normalDone, riskyDone := make(chan struct{}), make(chan struct{})
	var normalBorrowed, riskyBorrowed atomic.Int64
	normalLimit := opts.NormalLimit
	if normalLimit <= 0 {
		normalLimit = max(1, opts.Settings.FastWorkers+opts.Settings.MediumWorkers+opts.Settings.SlowWorkers)
	}
	permits := make(chan struct{}, normalLimit)
	safeEmit := func(r results.Result) { mu.Lock(); defer mu.Unlock(); emit(r) }
	phase := func(name string, jobs []scheduler.Job, workers int, ids []int) {
		if len(jobs) == 0 {
			return
		}
		began := time.Now()
		jobsCh := make(chan scheduler.Job)
		var wg sync.WaitGroup
		workers = min(max(1, workers), len(jobs))
		for i := 0; i < workers; i++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				for job := range jobsCh {
					if name != "risky" {
						permits <- struct{}{}
					}
					jobCPUs := ids
					if name == "risky" {
						select {
						case <-normalDone:
							jobCPUs = opts.CPUs
							riskyBorrowed.Add(1)
						default:
						}
					} else {
						select {
						case <-riskyDone:
							jobCPUs = opts.CPUs
							normalBorrowed.Add(1)
						default:
						}
					}
					safeEmit(test.Run(job.Path, test.Options{Risky: name == "risky" || opts.LowPriority, Engine: opts.Engine, CPUs: jobCPUs, Timeout: opts.Timeout, Timings: opts.Timings}))
					if name != "risky" {
						<-permits
					}
				}
			}()
		}
		for _, job := range jobs {
			jobsCh <- job
		}
		close(jobsCh)
		wg.Wait()
		mu.Lock()
		stats.Phases = append(stats.Phases, Phase{Name: name, Tests: len(jobs), Workers: workers, Duration: time.Since(began)})
		mu.Unlock()
	}
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer close(riskyDone)
		phase("risky", plan.Risky, opts.Settings.RiskyWorkers, risk)
	}()
	// Keep the independently calibrated normal phases sequential. Running each
	// at its isolated optimum concurrently oversubscribes the same CPU pool.
	phase("fast", plan.Fast, opts.Settings.FastWorkers, normal)
	phase("medium", plan.Medium, opts.Settings.MediumWorkers, normal)
	phase("slow", plan.Slow, opts.Settings.SlowWorkers, normal)
	close(normalDone)
	wg.Wait()
	stats.NormalBorrowed = normalBorrowed.Load()
	stats.RiskyBorrowed = riskyBorrowed.Load()
	stats.Duration = time.Since(start)
	return stats
}

func resolve(config RunConfig) (*calibration.File, *calibration.Environment, Execution, error) {
	ids, err := cpus.Budget(config.CPUCount)
	if err != nil {
		return nil, nil, Execution{}, err
	}
	if len(ids) == 0 {
		return nil, nil, Execution{}, fmt.Errorf("no available CPUs")
	}
	file := config.CalibrationPath
	if file == "" {
		file = "runner-calibration.json"
	}
	var store *calibration.File
	if config.Prepared != nil {
		store = config.Prepared.store
	} else {
		store, err = calibration.Load(file)
		if err != nil {
			return nil, nil, Execution{}, err
		}
	}
	key := config.CalibrationKey
	if key == "" {
		key = cpus.MachineKey(ids)
	}
	env := store.Environment(key, len(ids))
	if env.OS != runtime.GOOS || env.CPUs != len(ids) {
		return nil, nil, Execution{}, fmt.Errorf("calibration key %q belongs to %s/%d CPUs; choose a different key for %s/%d", key, env.OS, env.CPUs, runtime.GOOS, len(ids))
	}
	settings := DefaultSettings(len(ids))
	if env.Settings != nil {
		settings = *env.Settings
	}
	if config.Workers > 0 {
		settings.FastWorkers = config.Workers
		settings.MediumWorkers = config.Workers
		settings.SlowWorkers = config.Workers
	}
	if config.RiskyWorkers > 0 {
		settings.RiskyWorkers = config.RiskyWorkers
	}
	if config.RiskyCPUs > 0 {
		settings.RiskyCPUs = config.RiskyCPUs
	}
	if settings.RiskyCPUs >= len(ids) && len(ids) > 1 {
		return nil, nil, Execution{}, fmt.Errorf("risky CPU count must leave at least one CPU for normal tests")
	}
	if runtime.GOOS != "linux" {
		log.Printf("%s; tuning worker concurrency only", cpus.Isolation())
	}
	engine := config.Engine
	if engine == "" {
		engine = test.ENGINE_LOCATION
	}
	if _, err := exec.LookPath(engine); err != nil {
		return nil, nil, Execution{}, fmt.Errorf("engine unavailable: %w", err)
	}
	return store, env, Execution{NormalLimit: config.Workers, Settings: settings, CPUs: ids, Engine: engine, Timeout: config.Timeout, Timings: config.Timings}, nil
}
