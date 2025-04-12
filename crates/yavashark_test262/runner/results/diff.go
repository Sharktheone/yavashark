package results

import (
	"fmt"
	"github.com/gofiber/fiber/v2/log"
	"path/filepath"
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
	prevRaw, err := LoadResults()
	if err != nil {
		return nil, err
	}

	prev := FromResults(prevRaw)

	return tr.ComputeDiff(prev), nil
}
func (tr *TestResults) ComputeDiff(other *TestResults) Diff {
	return tr.ComputeDiffRoot(other, "")

}

func (tr *TestResults) ComputeDiffRoot(other *TestResults, root string) Diff {

	diff := make(Diff)

	aggregated := make(AggregatedDiff, max(len(tr.TestResults), len(other.TestResults)))

	for _, res := range tr.TestResults {
		path := fixPath(res.Path, root)
		log.Debug(path)
		aggregated[path] = DiffItem{
			own: &res,
		}
	}

	log.Debug("Aggregate other:")

	for _, res := range other.TestResults {
		if item, ok := aggregated[res.Path]; ok {
			path := fixPath(res.Path, root)
			log.Debug(path, res.Path)
			aggregated[path] = DiffItem{
				own:   item.own,
				other: &res,
			}
		}
	}

	for _, res := range aggregated {
		if res.own == nil || res.other == nil {
			log.Debug("Skipping nil result")
			continue
		}
		log.Debug("Diffing:", res.own.Path, res.other.Path)

		if res.own.Status != res.other.Status {
			log.Debug("Diff found:", res.own.Path, res.other.Path)
			d := TestDiff{To: res.own.Status, From: res.other.Status}

			if item, ok := diff[d]; ok {
				diff[d] = append(item, res)
			} else {
				diff[d] = []DiffItem{res}
			}
		}
	}

	diff.Sort()

	return diff
}

func fixPath(path, root string) string {
	path, err := filepath.Rel(root, path)
	if err != nil {
		return strings.TrimPrefix(path, root)
	}

	return path
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
			fmt.Printf("Diff: %s -> %s: %s\n", k.From, k.To, item.own.Path)
		}
	}

	print("\n\n\n")
}

func (d *Diff) PrintGrouped() {
	for k, v := range *d {
		fmt.Printf("Diff: %s -> %s\n", k.From, k.To)
		for _, item := range v {
			fmt.Printf("  - %s\n", item.own.Path)
		}
	}

	print("\n\n\n")
}

func (d *Diff) PrintDiffFilter(filter []TestDiff) {
	for _, f := range filter {
		if v, ok := (*d)[f]; ok {
			for _, item := range v {
				fmt.Printf("Diff: %s -> %s: %s\n", f.From, f.To, item.own.Path)
			}
		}
	}

	print("\n\n\n")
}

func (d *Diff) PrintGroupedFilter(filter []TestDiff) {
	for _, f := range filter {
		if v, ok := (*d)[f]; ok {
			fmt.Printf("Diff: %s -> %s\n", f.From, f.To)
			for _, item := range v {
				fmt.Printf("  - %s\n", item.own.Path)
			}
		}
	}

	print("\n\n\n")
}
