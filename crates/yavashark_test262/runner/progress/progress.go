package progress

import (
	"fmt"
	"sync"
	"sync/atomic"
	"yavashark_test262_runner/status"
)

const (
	ColorGreen   = "\033[32m"
	ColorRed     = "\033[31m"
	ColorYellow  = "\033[33m"
	ColorBlue    = "\033[34m"
	ColorMagenta = "\033[35m"
	ColorCyan    = "\033[36m"
	ColorReset   = "\033[0m"
	ColorGray    = "\033[90m"
)

type ProgressTracker struct {
	mu sync.Mutex

	passed              atomic.Uint32
	failed              atomic.Uint32
	skipped             atomic.Uint32
	timeout             atomic.Uint32
	crashed             atomic.Uint32
	parseError          atomic.Uint32
	parseSuccessError   atomic.Uint32
	notImplemented      atomic.Uint32
	runnerError         atomic.Uint32
	total               atomic.Uint32
	lastPrintedProgress atomic.Uint32

	totalTests uint32
}

func NewProgressTracker(totalTests uint32) *ProgressTracker {
	return &ProgressTracker{
		totalTests: totalTests,
	}
}

func (pt *ProgressTracker) Add(s status.Status) {
	switch s {
	case status.PASS:
		pt.passed.Add(1)
	case status.FAIL:
		pt.failed.Add(1)
	case status.SKIP:
		pt.skipped.Add(1)
	case status.TIMEOUT:
		pt.timeout.Add(1)
	case status.CRASH:
		pt.crashed.Add(1)
	case status.PARSE_ERROR:
		pt.parseError.Add(1)
	case status.PARSE_SUCCESS_ERROR:
		pt.parseSuccessError.Add(1)
	case status.NOT_IMPLEMENTED:
		pt.notImplemented.Add(1)
	case status.RUNNER_ERROR:
		pt.runnerError.Add(1)
	}

	current := pt.total.Add(1)
	pt.updateProgress(current)
}

func (pt *ProgressTracker) updateProgress(current uint32) {
	lastPrinted := pt.lastPrintedProgress.Load()

	threshold := uint32(100)
	if pt.totalTests > 0 {
		percentThreshold := pt.totalTests / 50 // 2%
		if percentThreshold > threshold {
			threshold = percentThreshold
		}
	}

	if current-lastPrinted >= threshold || current == pt.totalTests {
		if pt.lastPrintedProgress.CompareAndSwap(lastPrinted, current) {
			pt.printProgressBar(current)
		}
	}
}

func (pt *ProgressTracker) printProgressBar(current uint32) {
	passed := pt.passed.Load()
	failed := pt.failed.Load()
	skipped := pt.skipped.Load()
	timeout := pt.timeout.Load()
	crashed := pt.crashed.Load()

	barWidth := 50
	passedWidth := int(float64(passed) / float64(current) * float64(barWidth))
	failedWidth := int(float64(failed) / float64(current) * float64(barWidth))
	skippedWidth := int(float64(skipped) / float64(current) * float64(barWidth))
	timeoutWidth := int(float64(timeout) / float64(current) * float64(barWidth))
	crashedWidth := int(float64(crashed) / float64(current) * float64(barWidth))
	otherWidth := barWidth - passedWidth - failedWidth - skippedWidth - timeoutWidth - crashedWidth

	bar := ""
	if passedWidth > 0 {
		bar += ColorGreen + repeatChar("█", passedWidth) + ColorReset
	}
	if failedWidth > 0 {
		bar += ColorRed + repeatChar("█", failedWidth) + ColorReset
	}
	if timeoutWidth > 0 {
		bar += ColorYellow + repeatChar("█", timeoutWidth) + ColorReset
	}
	if crashedWidth > 0 {
		bar += ColorMagenta + repeatChar("█", crashedWidth) + ColorReset
	}
	if skippedWidth > 0 {
		bar += ColorCyan + repeatChar("█", skippedWidth) + ColorReset
	}
	if otherWidth > 0 {
		bar += ColorGray + repeatChar("░", otherWidth) + ColorReset
	}

	percentage := float64(current) / float64(pt.totalTests) * 100

	fmt.Printf("\r[%s] %3d%% (%d/%d) | %sP:%d%s %sF:%d%s %sT:%d%s %sC:%d%s %sS:%d%s",
		bar,
		int(percentage),
		current,
		pt.totalTests,
		ColorGreen, passed, ColorReset,
		ColorRed, failed, ColorReset,
		ColorYellow, timeout, ColorReset,
		ColorMagenta, crashed, ColorReset,
		ColorCyan, skipped, ColorReset,
	)
}

func (pt *ProgressTracker) GetStats() (passed, failed, skipped, timeout, crashed, parseError, parseSuccessError, notImplemented, runnerError, total uint32) {
	return pt.passed.Load(), pt.failed.Load(), pt.skipped.Load(), pt.timeout.Load(), pt.crashed.Load(),
		pt.parseError.Load(), pt.parseSuccessError.Load(), pt.notImplemented.Load(), pt.runnerError.Load(),
		pt.total.Load()
}

func repeatChar(char string, count int) string {
	result := ""
	for i := 0; i < count; i++ {
		result += char
	}
	return result
}
