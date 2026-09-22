//go:build linux || darwin

package test

import (
	"os"
	"runtime"
	"syscall"
)

func peakMemory(state *os.ProcessState) uint64 {
	usage, ok := state.SysUsage().(*syscall.Rusage)
	if !ok || usage.Maxrss < 0 {
		return 0
	}
	value := uint64(usage.Maxrss)
	if runtime.GOOS == "darwin" {
		value /= 1024
	}
	return value
}
