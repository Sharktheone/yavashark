package main

import (
	"context"
	"os/exec"
	"strings"
	"time"
	"yavashark_test262_runner/status"
)

const (
	ENGINE_LOCATION = "../../target/release/yavashark_test262"

	TIMEOUT = 4 * time.Second
)

func runTest(path string) Result {
	ctx, cancel := context.WithTimeout(context.Background(), TIMEOUT)
	defer cancel()

	cmd := exec.CommandContext(ctx, ENGINE_LOCATION, path)

	outRaw, err := cmd.CombinedOutput()

	out := string(outRaw)

	if ctx.Err() == context.DeadlineExceeded {
		return Result{
			Status: status.TIMEOUT,
			Msg:    "Test timed out",
			Path:   path,
		}
	}

	if err != nil {
		if strings.HasPrefix(out, "PARSE_ERROR") {
			return Result{
				Status: status.PARSE_ERROR,
				Msg:    out,
				Path:   path,
			}
		}

		if strings.Contains(out, "not yet implemented") && strings.Contains(out, "thread '") && strings.Contains(out, "' panicked at") {
			return Result{
				Status: status.NOT_IMPLEMENTED,
				Msg:    out,
				Path:   path,
			}
		}
		return Result{
			Status: status.CRASH,
			Msg:    out,
			Path:   path,
		}
	}

	if strings.HasPrefix(out, "PASS") {
		return Result{
			Status: status.PASS,
			Msg:    out,
			Path:   path,
		}
	} else if strings.HasPrefix(out, "FAIL") {
		return Result{
			Status: status.FAIL,
			Msg:    out,
			Path:   path,
		}
	}

	return Result{
		Status: status.CRASH,
		Msg:    out,
		Path:   path,
	}
}
