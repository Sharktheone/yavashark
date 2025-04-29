package test

import (
	"context"
	"errors"
	"os/exec"
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
	ctx, cancel := context.WithTimeout(context.Background(), TIMEOUT)
	defer cancel()

	cmd := exec.CommandContext(ctx, ENGINE_LOCATION, path)

	outRaw, err := cmd.CombinedOutput()

	out := string(outRaw)

	if errors.Is(ctx.Err(), context.DeadlineExceeded) {
		return results.Result{
			Status: status.TIMEOUT,
			Msg:    "Test timed out",
			Path:   path,
		}
	}

	if err != nil {
		if strings.HasPrefix(out, "PARSE_ERROR") {
			return results.Result{
				Status: status.PARSE_ERROR,
				Msg:    out,
				Path:   path,
			}
		}

		if strings.Contains(out, "not yet implemented") && strings.Contains(out, "thread '") && strings.Contains(out, "' panicked at") {
			return results.Result{
				Status: status.NOT_IMPLEMENTED,
				Msg:    out,
				Path:   path,
			}
		}
		return results.Result{
			Status: status.CRASH,
			Msg:    out,
			Path:   path,
		}
	}

	if strings.HasPrefix(out, "PASS") {
		return results.Result{
			Status: status.PASS,
			Msg:    out,
			Path:   path,
		}
	}

	if strings.HasPrefix(out, "FAIL") {
		return results.Result{
			Status: status.FAIL,
			Msg:    out,
			Path:   path,
		}
	}

	if strings.HasPrefix(out, "Test262:AsyncTestComplete") {
		return results.Result{
			Status: status.PASS,
			Msg:    out,
			Path:   path,
		}
	}

	if strings.HasPrefix(out, "Test262:AsyncTestFailure:") {
		return results.Result{
			Status: status.FAIL,
			Msg:    out,
			Path:   path,
		}
	}

	return results.Result{
		Status: status.CRASH,
		Msg:    out,
		Path:   path,
	}
}
