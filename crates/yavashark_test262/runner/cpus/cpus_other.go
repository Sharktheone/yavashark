//go:build !linux

package cpus

import (
	"os/exec"
	"runtime"
	"strconv"
)

func Isolation() string  { return "shared CPUs: hard affinity unavailable on this OS" }
func core(id int) string { return strconv.Itoa(id) }
func available() ([]int, error) {
	ids := make([]int, runtime.NumCPU())
	for i := range ids {
		ids[i] = i
	}
	return ids, nil
}
func Start(cmd *exec.Cmd, ids []int) error { return cmd.Start() }

func LaunchArgs(engine, path string, risky bool) (string, []string) {
	if runtime.GOOS == "darwin" && risky {
		return "/usr/bin/nice", []string{"-n", "10", engine, path}
	}
	return engine, []string{path}
}
