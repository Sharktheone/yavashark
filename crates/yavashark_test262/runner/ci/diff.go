package ci

import (
	"encoding/json"
	"os"
	"yavashark_test262_runner/results"
)

func loadPrevCi(path string) (*results.TestResults, error) {
	contents, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}

		return nil, err
	}

	var resultsCI []results.CIResult
	err = json.Unmarshal(contents, &resultsCI)

	if err != nil {
		return nil, err
	}

	res := results.ConvertResultsFromCI(resultsCI)

	return results.FromResults(res), nil
}
