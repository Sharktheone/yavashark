package main

import (
	"encoding/json"
	"os"
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

func writeResults(results []Result) error {
	out, err := json.Marshal(results)
	if err != nil {
		return err
	}

	err = os.WriteFile(RESULT_PATH, out, 0644)

	return nil
}
