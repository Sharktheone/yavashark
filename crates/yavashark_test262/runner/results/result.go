package results

import (
	"encoding/json"
	"log"
	"os"
	"path/filepath"
	"time"
	"yavashark_test262_runner/status"
)

const (
	RESULT_PATH = "results.json"
)

type Result struct {
	Status   status.Status `json:"status"`
	Msg      string        `json:"msg"`
	Path     string        `json:"path"`
	MemoryKB uint64        `json:"memory_kb"`
	Duration time.Duration `json:"duration"`
}

type CIResult struct {
	Status status.CIStatus `json:"s"`
	Path   string          `json:"p"`
}

func writeResults(results []Result) error {
	return writeResultsPath(results, RESULT_PATH)
}

func LoadResults() ([]Result, error) {
	return loadResultsPath(RESULT_PATH)
}

func writeResultsPath(results []Result, path string) error {
	out, err := json.Marshal(results)
	if err != nil {
		return err
	}

	err = os.WriteFile(path, out, 0644)

	return nil
}

func loadResultsPath(path string) ([]Result, error) {
	contents, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, err
	}

	var results []Result

	err = json.Unmarshal(contents, &results)

	return results, err
}

func ConvertResultsToCI(results []Result, root string) []CIResult {
	ciResults := make([]CIResult, len(results))
	for i, res := range results {
		path, err := filepath.Rel(root, res.Path)
		if err != nil {
			path = res.Path
		}

		ciResults[i] = CIResult{
			Status: res.Status.ToCIStatus(),
			Path:   path,
		}
	}

	return ciResults
}

func ConvertResultsFromCI(results []CIResult) []Result {
	r := make([]Result, len(results))

	for i, res := range results {
		r[i] = Result{
			Status: res.Status.ToStatus(),
			Msg:    "",
			Path:   res.Path,
		}
	}

	return r
}

func WriteCIResultsPath(results []Result, path string, root string) error {
	ciResults := ConvertResultsToCI(results, root)

	out, err := json.Marshal(ciResults)
	if err != nil {
		return err
	}

	log.Printf("writing CI results to %s", path)

	err = os.WriteFile(path, out, 0644)

	return nil
}
