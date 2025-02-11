package main

import (
	"encoding/json"
	"log"
	"os"
	"path/filepath"
	"yavashark_test262_runner/status"
)

const (
	RESULT_PATH = "results.json"
)

type Result struct {
	Status status.Status `json:"status"`
	Msg    string        `json:"msg"`
	Path   string        `json:"path"`
}

type CIResult struct {
	Status status.CIStatus `json:"status"`
	Path   string          `json:"path"`
}

func writeResults(results []Result) error {
	return writeResultsPath(results, RESULT_PATH)
}

func writeResultsPath(results []Result, path string) error {
	out, err := json.Marshal(results)
	if err != nil {
		return err
	}

	err = os.WriteFile(path, out, 0644)

	return nil
}

func convertResultsToCI(results []Result, root string) []CIResult {
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

func writeCIResultsPath(results []Result, path string, root string) error {
	ciResults := convertResultsToCI(results, root)

	out, err := json.Marshal(ciResults)
	if err != nil {
		return err
	}

	log.Printf("writing CI results to %s", path)

	err = os.WriteFile(path, out, 0644)

	return nil
}
