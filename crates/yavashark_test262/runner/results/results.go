package results

import (
	"fmt"
	"log"
	"sort"
	"time"
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
	ColorBold    = "\033[1m"
	ColorDim     = "\033[2m"
)

type TestResults struct {
	TestResults       []Result
	Passed            uint32
	Failed            uint32
	Skipped           uint32
	NotImplemented    uint32
	RunnerError       uint32
	Crashed           uint32
	Timeout           uint32
	ParseError        uint32
	ParseSuccessError uint32
	Total             uint32
}

func New(num uint32) *TestResults {
	return &TestResults{
		TestResults: make([]Result, 0, num),
	}
}

func FromResults(results []Result) *TestResults {
	tr := &TestResults{
		TestResults: results,
	}

	tr.analyze()

	return tr
}

func (tr *TestResults) analyze() {
	for _, res := range tr.TestResults {
		tr.Total++
		switch res.Status {
		case status.PASS:
			tr.Passed++
		case status.FAIL:
			tr.Failed++
		case status.SKIP:
			tr.Skipped++
		case status.NOT_IMPLEMENTED:
			tr.NotImplemented++
		case status.RUNNER_ERROR:
			tr.RunnerError++
		case status.CRASH:
			tr.Crashed++
		case status.PARSE_ERROR:
			tr.ParseError++
		case status.PARSE_SUCCESS_ERROR:
			tr.ParseSuccessError++
		case status.TIMEOUT:
			tr.Timeout++
		}
	}
}

func (tr *TestResults) Add(res Result) {
	tr.Total++
	switch res.Status {
	case status.PASS:
		tr.Passed++
	case status.FAIL:
		tr.Failed++
	case status.SKIP:
		tr.Skipped++
	case status.NOT_IMPLEMENTED:
		tr.NotImplemented++
	case status.RUNNER_ERROR:
		tr.RunnerError++
	case status.CRASH:
		tr.Crashed++
	case status.PARSE_ERROR:
		tr.ParseError++
	case status.PARSE_SUCCESS_ERROR:
		tr.ParseSuccessError++
	case status.TIMEOUT:
		tr.Timeout++
	}

	tr.TestResults = append(tr.TestResults, res)
}

func (tr *TestResults) PrintResults(showStats bool) {
	printRes("Passed", tr.Passed, tr.Total)
	printRes("Failed", tr.Failed, tr.Total)
	printRes("Skipped", tr.Skipped, tr.Total)
	printRes("Not Implemented", tr.NotImplemented, tr.Total)
	printRes("Runner Error", tr.RunnerError, tr.Total)
	printRes("Crashed", tr.Crashed, tr.Total)
	printRes("Timeout", tr.Timeout, tr.Total)
	printRes("Parse Error", tr.ParseError, tr.Total)
	printRes("Parse Success Error", tr.ParseSuccessError, tr.Total)
	fmt.Printf("Total: %d\n", tr.Total)

	printRes("Passed (no parse)", tr.Passed, tr.Total-(tr.ParseError+tr.ParseSuccessError))
	fmt.Printf("Total (no parse): %d\n", tr.Total-(tr.ParseError+tr.ParseSuccessError))

	printRes("Passed (skipped)", tr.Passed, tr.Total-tr.Skipped)
	fmt.Printf("Total (skipped): %d\n", tr.Total-tr.Skipped)

	printRes("Passed (skip, no-parse)", tr.Passed, tr.Total-(tr.Skipped+tr.ParseError+tr.ParseSuccessError))
	fmt.Printf("Total (skip, no-parse): %d\n", tr.Total-(tr.Skipped+tr.ParseError+tr.ParseSuccessError))

	if showStats {
		tr.PrintMemoryStats()
	}
}

func formatMemory(kb uint64) string {
	if kb >= 1024*1024 {
		gb := float64(kb) / (1024 * 1024)
		return fmt.Sprintf("%.2f GB", gb)
	} else if kb >= 1024 {
		mb := float64(kb) / 1024
		return fmt.Sprintf("%.2f MB", mb)
	}
	return fmt.Sprintf("%d KB", kb)
}

func (tr *TestResults) PrintMemoryStats() {
	if len(tr.TestResults) == 0 {
		return
	}

	results := make([]Result, len(tr.TestResults))
	copy(results, tr.TestResults)

	sort.Slice(results, func(i, j int) bool {
		return results[i].MemoryKB > results[j].MemoryKB
	})

	fmt.Printf("\n=== Top 10 Tests by Memory Usage ===\n")
	limit := 10
	if len(results) < limit {
		limit = len(results)
	}

	for i := 0; i < limit; i++ {
		result := results[i]
		fmt.Printf("%2d. %s - %s - %v - %s\n",
			i+1,
			result.Path,
			formatMemory(result.MemoryKB),
			result.Duration.Round(time.Millisecond),
			result.Status.String())
	}

	var totalMemory uint64
	var totalDuration time.Duration
	var maxMemory uint64
	var maxDuration time.Duration

	for _, result := range tr.TestResults {
		totalMemory += result.MemoryKB
		totalDuration += result.Duration
		if result.MemoryKB > maxMemory {
			maxMemory = result.MemoryKB
		}
		if result.Duration > maxDuration {
			maxDuration = result.Duration
		}
	}

	avgMemory := totalMemory / uint64(len(tr.TestResults))
	avgDuration := totalDuration / time.Duration(len(tr.TestResults))

	fmt.Printf("\n=== Memory and Timing Statistics ===\n")
	fmt.Printf("Average memory usage: %s\n", formatMemory(avgMemory))
	fmt.Printf("Maximum memory usage: %s\n", formatMemory(maxMemory))
	fmt.Printf("Total memory used: %s\n", formatMemory(totalMemory))
	fmt.Printf("Average test duration: %v\n", avgDuration.Round(time.Millisecond))
	fmt.Printf("Maximum test duration: %v\n", maxDuration.Round(time.Millisecond))
	fmt.Printf("Total test time: %v\n", totalDuration.Round(time.Millisecond))
}

func printRes(name string, n uint32, total uint32) {
	fmt.Printf("%s: %d, %f%%\n", name, n, float64(n)/float64(total)*100)
}

func (tr *TestResults) ComparePrev() error {
	prevRaw, err := LoadResults()
	if err != nil {
		return err
	}
	if prevRaw == nil {
		return nil
	}

	prev := FromResults(prevRaw)

	tr.Compare(prev)

	return nil
}

func (tr *TestResults) Compare(other *TestResults) {
	printDiff("Passed", tr.Passed, other.Passed, tr.Total)
	printDiff("Failed", tr.Failed, other.Failed, tr.Total)
	printDiff("Skipped", tr.Skipped, other.Skipped, tr.Total)
	printDiff("Not Implemented", tr.NotImplemented, other.NotImplemented, tr.Total)
	printDiff("Runner Error", tr.RunnerError, other.RunnerError, tr.Total)
	printDiff("Crashed", tr.Crashed, other.Crashed, tr.Total)
	printDiff("Timeout", tr.Timeout, other.Timeout, tr.Total)
	printDiff("Parse Error", tr.ParseError, other.ParseError, tr.Total)
	printDiff("Total", tr.Total, other.Total, tr.Total)
}

func (tr *TestResults) PrintResultsWithDiff(other *TestResults) {
	fmt.Printf("\n%s=== Test Results Summary ===%s\n\n", ColorBold, ColorReset)

	fmt.Printf("%sCurrent Run:%s\n", ColorBold, ColorReset)
	tr.printResultsLine("Passed", tr.Passed, tr.Total, other.Passed)
	tr.printResultsLine("Failed", tr.Failed, tr.Total, other.Failed)
	tr.printResultsLine("Timeout", tr.Timeout, tr.Total, other.Timeout)
	tr.printResultsLine("Crashed", tr.Crashed, tr.Total, other.Crashed)
	tr.printResultsLine("Skipped", tr.Skipped, tr.Total, other.Skipped)
	tr.printResultsLine("Not Implemented", tr.NotImplemented, tr.Total, other.NotImplemented)
	tr.printResultsLine("Runner Error", tr.RunnerError, tr.Total, other.RunnerError)
	tr.printResultsLine("Parse Error", tr.ParseError, tr.Total, other.ParseError)
	tr.printResultsLine("Parse Success Error", tr.ParseSuccessError, tr.Total, other.ParseSuccessError)

	fmt.Printf("\n%sTotal: %d%s\n", ColorBold, tr.Total, ColorReset)

	fmt.Printf("\n%s=== Net Changes ===%s\n", ColorBold, ColorReset)
	netPassed := int32(tr.Passed) - int32(other.Passed)
	netFailed := int32(tr.Failed) - int32(other.Failed)
	netTimeout := int32(tr.Timeout) - int32(other.Timeout)
	netCrashed := int32(tr.Crashed) - int32(other.Crashed)

	if netPassed != 0 {
		if netPassed > 0 {
			fmt.Printf("%s✓ Passed: +%d%s (gained)\n", ColorGreen, netPassed, ColorReset)
		} else {
			fmt.Printf("%s✗ Passed: %d%s (lost)\n", ColorRed, netPassed, ColorReset)
		}
	}

	if netFailed != 0 {
		if netFailed > 0 {
			fmt.Printf("%s✗ Failed: +%d%s (gained)\n", ColorRed, netFailed, ColorReset)
		} else {
			fmt.Printf("%s✓ Failed: %d%s (improved)\n", ColorGreen, netFailed, ColorReset)
		}
	}

	if netTimeout != 0 {
		if netTimeout > 0 {
			fmt.Printf("%s⏱ Timeout: +%d%s (gained)\n", ColorYellow, netTimeout, ColorReset)
		} else {
			fmt.Printf("%s✓ Timeout: %d%s (improved)\n", ColorGreen, netTimeout, ColorReset)
		}
	}

	if netCrashed != 0 {
		if netCrashed > 0 {
			fmt.Printf("%s💥 Crashed: +%d%s (gained)\n", ColorMagenta, netCrashed, ColorReset)
		} else {
			fmt.Printf("%s✓ Crashed: %d%s (improved)\n", ColorGreen, netCrashed, ColorReset)
		}
	}

	// Overall summary
	fmt.Printf("\n%s=== Overall Summary ===%s\n", ColorBold, ColorReset)
	totalChanges := abs(netPassed) + abs(netFailed) + abs(netTimeout) + abs(netCrashed)
	if totalChanges > 0 {
		fmt.Printf("Total test status changes: %d\n", totalChanges)

		passedGained := 0
		if netPassed > 0 {
			passedGained = int(netPassed)
		}
		failedLost := 0
		if netFailed < 0 {
			failedLost = int(-netFailed)
		}
		improvements := passedGained + failedLost

		if improvements > 0 {
			fmt.Printf("%s↑ Improvements: %d%s\n", ColorGreen, improvements, ColorReset)
		}

		failedGained := 0
		if netFailed > 0 {
			failedGained = int(netFailed)
		}
		passedLost := 0
		if netPassed < 0 {
			passedLost = int(-netPassed)
		}
		regressions := failedGained + passedLost

		if regressions > 0 {
			fmt.Printf("%s↓ Regressions: %d%s\n", ColorRed, regressions, ColorReset)
		}
	} else {
		fmt.Printf("%sNo changes from previous run%s\n", ColorDim, ColorReset)
	}
}

func (tr *TestResults) printResultsLine(name string, current, total, previous uint32) {
	percentage := float64(current) / float64(total) * 100
	diff := int32(current) - int32(previous)

	var diffStr string
	if diff > 0 {
		diffStr = fmt.Sprintf(" %s(+%d)%s", ColorGreen, diff, ColorReset)
	} else if diff < 0 {
		diffStr = fmt.Sprintf(" %s(%d)%s", ColorRed, diff, ColorReset)
	}

	fmt.Printf("  %s: %d (%.2f%%)%s\n", name, current, percentage, diffStr)
}

func abs(n int32) int32 {
	if n < 0 {
		return -n
	}
	return n
}

func printDiff(name string, n1 uint32, n2 uint32, total uint32) {
	n := int32(n1) - int32(n2)

	perc := float64(n) / float64(total) * 100
	fmt.Printf("%s: %+d, %+f%%\n", name, n, perc)
}

func (tr *TestResults) Write() {
	err := writeResults(tr.TestResults)

	if err != nil {
		log.Fatalf("Failed to write results: %v", err)
	}
}
