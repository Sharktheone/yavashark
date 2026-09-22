// Package cpus describes the engine CPU budget. Workers are independent of it.
package cpus

import (
	"fmt"
	"os"
	"runtime"
	"sort"
	"strings"
)

func Budget(n int) ([]int, error) {
	ids, err := available()
	if err != nil {
		return nil, err
	}
	if n < 0 || n > len(ids) {
		return nil, fmt.Errorf("CPU budget %d exceeds available CPUs (%d)", n, len(ids))
	}
	if n > 0 {
		ids = ids[:n]
	}
	return ids, nil
}

// Split keeps SMT siblings in the same pool. A requested count may round up.
func Split(ids []int, risky int) (normal, risk []int) {
	if risky <= 0 || len(ids) < 2 {
		return append([]int(nil), ids...), nil
	}
	groups := map[string][]int{}
	var keys []string
	for _, id := range ids {
		key := core(id)
		if _, ok := groups[key]; !ok {
			keys = append(keys, key)
		}
		groups[key] = append(groups[key], id)
	}
	if len(keys) < 2 {
		return append([]int(nil), ids...), nil
	}
	selected := map[int]bool{}
	for i := len(keys) - 1; i > 0 && len(risk) < risky; i-- {
		for _, id := range groups[keys[i]] {
			risk = append(risk, id)
			selected[id] = true
		}
	}
	for _, id := range ids {
		if !selected[id] {
			normal = append(normal, id)
		}
	}
	sort.Ints(risk)
	return
}

func MachineKey(ids []int) string {
	host, _ := os.Hostname()
	return fmt.Sprintf("%s/%s/%s/%d", runtime.GOOS, runtime.GOARCH, host, len(ids))
}

func Description(ids []int) string {
	return fmt.Sprintf("%v (%s)", ids, strings.TrimSpace(Isolation()))
}
