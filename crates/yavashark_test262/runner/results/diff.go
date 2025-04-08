package results

import (
	"fmt"
	"log"
	"sort"
	"strings"
	"yavashark_test262_runner/status"
)

type TestDiff struct {
	From status.Status
	To   status.Status
}

type DiffItem struct {
	own   *Result
	other *Result
}

type Diff map[TestDiff][]DiffItem
type AggregatedDiff map[string]DiffItem

func ParseFilter(filter string) ([]TestDiff, error) {
	var result []TestDiff

	for _, s := range strings.Split(filter, ",") {
		parts := strings.Split(s, "->")
		if len(parts) != 2 {
			return nil, fmt.Errorf("invalid filter format: %s", s)
		}

		from, err := status.ParseStatus(strings.TrimSpace(parts[0]))
		if err != nil {
			return nil, err
		}
		to, err := status.ParseStatus(strings.TrimSpace(parts[1]))
		if err != nil {
			return nil, err
		}

		result = append(result, TestDiff{From: from, To: to})
	}

	return result, nil
}

func (tr *TestResults) ComputeDiffPrev() (Diff, error) {
	prevRaw, err := loadResults()
	if err != nil {
		return nil, err
	}

	prev := fromResults(prevRaw)

	return tr.ComputeDiff(prev)
}

func (tr *TestResults) ComputeDiff(other *TestResults) (Diff, error) {
	diff := make(Diff)

	aggregated := make(AggregatedDiff, max(len(tr.TestResults), len(other.TestResults)))

	for _, res := range tr.TestResults {
		aggregated[res.Path] = DiffItem{
			own: &res,
		}
	}

	for _, res := range other.TestResults {
		if item, ok := aggregated[res.Path]; ok {
			item.other = &res
		}
	}

	for _, res := range aggregated {
		if res.own == nil || res.other == nil {
			continue
		}

		if res.own.Status != res.other.Status {
			d := TestDiff{From: res.own.Status, To: res.other.Status}

			if item, ok := diff[d]; ok {
				item = append(item, res)
			} else {
				diff[d] = []DiffItem{res}
			}
		}
	}

	diff.Sort()

	return diff, nil
}

func (d *Diff) Sort() {
	for _, v := range *d {
		sort.Slice(v, func(i, j int) bool {
			return v[i].own.Path < v[j].own.Path
		})
	}
}

func (d *Diff) PrintDiff() {
	for k, v := range *d {
		for _, item := range v {
			log.Printf("Diff: %s -> %s: %s\n", k.From, k.To, item.own.Path)
		}
	}
}

func (d *Diff) PrintGrouped() {
	for k, v := range *d {
		log.Printf("Diff: %s -> %s\n", k.From, k.To)
		for _, item := range v {
			log.Printf("  - %s\n", item.own.Path)
		}
	}
}

func (d *Diff) PrintDiffFilter(filter []TestDiff) {
	for _, f := range filter {
		if v, ok := (*d)[f]; ok {
			for _, item := range v {
				log.Printf("Diff: %s -> %s: %s\n", f.From, f.To, item.own.Path)
			}
		}
	}
}

func (d *Diff) PrintGroupedFilter(filter []TestDiff) {
	for _, f := range filter {
		if v, ok := (*d)[f]; ok {
			log.Printf("Diff: %s -> %s\n", f.From, f.To)
			for _, item := range v {
				log.Printf("  - %s\n", item.own.Path)
			}
		}
	}
}
