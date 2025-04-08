package results

import (
	"fmt"
	"log"
	"yavashark_test262_runner/status"
)

type TestResults struct {
	TestResults    []Result
	Passed         uint32
	Failed         uint32
	Skipped        uint32
	NotImplemented uint32
	RunnerError    uint32
	Crashed        uint32
	Timeout        uint32
	ParseError     uint32
	Total          uint32
}

func New(num uint32) *TestResults {
	return &TestResults{
		TestResults: make([]Result, 0, num),
	}
}

func fromResults(results []Result) *TestResults {
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
	case status.TIMEOUT:
		tr.Timeout++
	}

	tr.TestResults = append(tr.TestResults, res)
}

func (tr *TestResults) PrintResults() {
	printRes("Passed", tr.Passed, tr.Total)
	printRes("Failed", tr.Failed, tr.Total)
	printRes("Skipped", tr.Skipped, tr.Total)
	printRes("Not Implemented", tr.NotImplemented, tr.Total)
	printRes("Runner Error", tr.RunnerError, tr.Total)
	printRes("Crashed", tr.Crashed, tr.Total)
	printRes("Timeout", tr.Timeout, tr.Total)
	printRes("Parse Error", tr.ParseError, tr.Total)
	fmt.Printf("Total: %d\n", tr.Total)

	printRes("Passed (no parse)", tr.Passed, tr.Total-tr.ParseError)
	fmt.Printf("Total (no parse): %d\n", tr.Total-tr.ParseError)
}

func printRes(name string, n uint32, total uint32) {
	fmt.Printf("%s: %d, %f%%\n", name, n, float64(n)/float64(total)*100)
}

func (tr *TestResults) ComparePrev() error {
	prevRaw, err := loadResults()
	if err != nil {
		return err
	}
	if prevRaw == nil {
		return nil
	}

	prev := fromResults(prevRaw)

	printDiff("Passed", tr.Passed, prev.Passed, tr.Total)
	printDiff("Failed", tr.Failed, prev.Failed, tr.Total)
	printDiff("Skipped", tr.Skipped, prev.Skipped, tr.Total)
	printDiff("Not Implemented", tr.NotImplemented, prev.NotImplemented, tr.Total)
	printDiff("Runner Error", tr.RunnerError, prev.RunnerError, tr.Total)
	printDiff("Crashed", tr.Crashed, prev.Crashed, tr.Total)
	printDiff("Timeout", tr.Timeout, prev.Timeout, tr.Total)
	printDiff("Parse Error", tr.ParseError, prev.ParseError, tr.Total)
	printDiff("Total", tr.Total, prev.Total, tr.Total)

	return nil
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
