package test

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
)

const (
	ENGINE_LOCATION = "../../target/release/yavashark_test262"

	TIMEOUT = 30 * time.Second
)

func RunTest(path string) results.Result {
	startTime := time.Now()

	ctx, cancel := context.WithTimeout(context.Background(), TIMEOUT)
	defer cancel()

	cmd := exec.CommandContext(ctx, ENGINE_LOCATION, path)

	var b bytes.Buffer
	cmd.Stdout = &b
	cmd.Stderr = &b

	err := cmd.Start()
	if err != nil {
		return results.Result{
			Status:   status.RUNNER_ERROR,
			Msg:      fmt.Sprintf("Failed to start process: %v", err),
			Path:     path,
			MemoryKB: 0,
			Duration: time.Since(startTime),
		}
	}

	var peakMemoryKB uint64
	done := make(chan bool)

	go func() {
		ticker := time.NewTicker(10 * time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-done:
				return
			case <-ticker.C:
				if cmd.Process != nil {
					memKB := getProcessMemoryKB(cmd.Process.Pid)
					if memKB > peakMemoryKB {
						peakMemoryKB = memKB
					}
				}
			}
		}
	}()

	waitErr := cmd.Wait()

	close(done)

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

		if strings.Contains(out, "not yet implemented") && strings.Contains(out, "thread '") && strings.Contains(out, "' panicked at") {
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

func getProcessMemoryKB(pid int) uint64 {
	statusFile := fmt.Sprintf("/proc/%d/status", pid)
	data, err := os.ReadFile(statusFile)
	if err != nil {
		return 0
	}

	lines := strings.Split(string(data), "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "VmRSS:") {
			fields := strings.Fields(line)
			if len(fields) >= 2 {
				memKB, err := strconv.ParseUint(fields[1], 10, 64)
				if err == nil {
					return memKB
				}
			}
		}
	}
	return 0
}
