package test

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
	"yavashark_test262_runner/cpus"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
	"yavashark_test262_runner/timing"
)

const (
	ENGINE_LOCATION = "../../target/release/yavashark_test262"

	DEFAULT_TIMEOUT = 30 * time.Second
)

var TestTimeout = DEFAULT_TIMEOUT

func SetTimeout(timeout time.Duration) {
	TestTimeout = timeout
}

type Options struct {
	Risky   bool
	Engine  string
	CPUs    []int
	Timeout time.Duration
	Timings bool
}

func RunTest(path string, timings bool) results.Result {
	return Run(path, Options{Timeout: TestTimeout, Timings: timings})
}

func Run(path string, options Options) (result results.Result) {
	startTime := time.Now()
	engine := options.Engine
	if engine == "" {
		engine = ENGINE_LOCATION
	}
	timeout := options.Timeout
	if timeout <= 0 {
		timeout = DEFAULT_TIMEOUT
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	executable, args := cpus.LaunchArgs(engine, path, options.Risky)
	cmd := exec.CommandContext(ctx, executable, args...)
	cmd.Env = append(os.Environ(), "YAVASHARK_TEST262_RUNNER=1")
	defer func() {
		if cmd.ProcessState != nil {
			result.CPUTime = cmd.ProcessState.UserTime() + cmd.ProcessState.SystemTime()
			result.MemoryKB = peakMemory(cmd.ProcessState)
		}
	}()

	var b bytes.Buffer
	cmd.Stdout = &b
	cmd.Stderr = &b

	err := cpus.Start(cmd, options.CPUs)
	if err != nil {
		return results.Result{
			Status:   status.RUNNER_ERROR,
			Msg:      fmt.Sprintf("Failed to start process: %v", err),
			Path:     path,
			MemoryKB: 0,
			Duration: time.Since(startTime),
		}
	}

	waitErr := cmd.Wait()
	var peakMemoryKB uint64

	duration := time.Since(startTime)

	out := b.String()

	if errors.Is(ctx.Err(), context.DeadlineExceeded) {
		return results.Result{
			Status:   status.TIMEOUT,
			Msg:      "Test timed out",
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if options.Timings {
		timing.ParseDurations(out)
	}

	// what the f... is this code btw?
	if waitErr != nil {
		if strings.HasPrefix(out, "PARSE_ERROR") {
			return results.Result{
				Status:   status.PARSE_ERROR,
				Msg:      out,
				Path:     path,
				MemoryKB: peakMemoryKB,
				Duration: duration,
			}
		}

		if strings.HasPrefix(out, "PARSE_SUCCESS_ERROR") {
			return results.Result{
				Status:   status.PARSE_SUCCESS_ERROR,
				Msg:      out,
				Path:     path,
				MemoryKB: peakMemoryKB,
				Duration: duration,
			}
		}

		if strings.Contains(out, "not yet implemented") && strings.Contains(out, "thread '") && strings.Contains(out, "panicked at") {
			return results.Result{
				Status:   status.NOT_IMPLEMENTED,
				Msg:      out,
				Path:     path,
				MemoryKB: peakMemoryKB,
				Duration: duration,
			}
		}
		return results.Result{
			Status:   status.CRASH,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if strings.HasPrefix(out, "PASS") {
		return results.Result{
			Status:   status.PASS,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if strings.HasPrefix(out, "FAIL") {
		return results.Result{
			Status:   status.FAIL,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if strings.HasPrefix(out, "Test262:AsyncTestComplete") {
		return results.Result{
			Status:   status.PASS,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if strings.HasPrefix(out, "Test262:AsyncTestFailure:") {
		return results.Result{
			Status:   status.FAIL,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	if strings.HasPrefix(out, "SKIP") {
		return results.Result{
			Status:   status.SKIP,
			Msg:      out,
			Path:     path,
			MemoryKB: peakMemoryKB,
			Duration: duration,
		}
	}

	return results.Result{
		Status:   status.CRASH,
		Msg:      out,
		Path:     path,
		MemoryKB: peakMemoryKB,
		Duration: duration,
	}
}
