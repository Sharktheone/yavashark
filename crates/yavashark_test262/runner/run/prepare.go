package run

import (
	"fmt"
	"slices"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/results"
)

// Prepared is immutable input data that local rebuilds can load while Cargo runs.
// It is consumed only by the matching run after compilation has succeeded.
type Prepared struct {
	paths          []string
	tests, skipped []string
	store          *calibration.File
	history        map[string]results.Result
}

func Prepare(paths []string, config RunConfig) (*Prepared, error) {
	tests, skipped, errs := Discover(paths, config.Skips)
	if len(errs) > 0 {
		return nil, fmt.Errorf("test discovery failed: %v", errs)
	}
	file := config.CalibrationPath
	if file == "" {
		file = "runner-calibration.json"
	}
	store, err := calibration.Load(file)
	if err != nil {
		return nil, err
	}
	file = config.HistoryPath
	if file == "" {
		file = "results.json"
	}
	history, err := calibration.History(file)
	if err != nil {
		return nil, err
	}
	return &Prepared{paths: slices.Clone(paths), tests: tests, skipped: skipped, store: store, history: history}, nil
}
func preparedInputs(paths []string, config RunConfig) (*Prepared, error) {
	if config.Prepared != nil {
		if !slices.Equal(paths, config.Prepared.paths) {
			return nil, fmt.Errorf("prepared inputs do not match requested paths")
		}
		return config.Prepared, nil
	}
	return Prepare(paths, config)
}
