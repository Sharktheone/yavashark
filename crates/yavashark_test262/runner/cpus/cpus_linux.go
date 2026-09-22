//go:build linux

package cpus

import (
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"sort"
	"strconv"
	"strings"

	"golang.org/x/sys/unix"
)

func Isolation() string { return "Linux affinity; SMT siblings kept together" }
func core(id int) string {
	base := fmt.Sprintf("/sys/devices/system/cpu/cpu%d/topology/", id)
	pkg, e1 := os.ReadFile(base + "physical_package_id")
	c, e2 := os.ReadFile(base + "core_id")
	if e1 != nil || e2 != nil {
		return strconv.Itoa(id)
	}
	return strings.TrimSpace(string(pkg)) + ":" + strings.TrimSpace(string(c))
}
func available() ([]int, error) {
	var mask unix.CPUSet
	if err := unix.SchedGetaffinity(0, &mask); err != nil {
		return nil, err
	}
	var ids []int
	for i := 0; i < 1024; i++ {
		if mask.IsSet(i) {
			ids = append(ids, i)
		}
	}
	// First hardware thread of each physical core, then SMT siblings.
	seen := map[string]int{}
	rank := map[int]int{}
	for _, id := range ids {
		k := core(id)
		rank[id] = seen[k]
		seen[k]++
	}
	sort.SliceStable(ids, func(i, j int) bool { return rank[ids[i]] < rank[ids[j]] })
	return ids, nil
}

// Start pins only the launching OS thread; fork/exec inherits its mask.
// Restoring the mask before unlocking avoids affecting other Go goroutines.
func Start(cmd *exec.Cmd, ids []int) error {
	if len(ids) == 0 {
		return cmd.Start()
	}
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()
	var previous, mask unix.CPUSet
	if err := unix.SchedGetaffinity(0, &previous); err != nil {
		return err
	}
	for _, id := range ids {
		mask.Set(id)
	}
	if err := unix.SchedSetaffinity(0, &mask); err != nil {
		return err
	}
	err := cmd.Start()
	if restore := unix.SchedSetaffinity(0, &previous); restore != nil {
		// A failed restore cannot safely return this thread to Go's pool.
		panic(fmt.Sprintf("restore runner CPU affinity: %v", restore))
	}
	return err
}

func LaunchArgs(engine, path string, risky bool) (string, []string) { return engine, []string{path} }
