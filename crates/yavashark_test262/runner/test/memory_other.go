//go:build !linux && !darwin

package test

import "os"

func peakMemory(state *os.ProcessState) uint64 { return 0 }
