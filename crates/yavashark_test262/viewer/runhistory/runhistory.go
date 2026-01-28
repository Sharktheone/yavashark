package runhistory

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"
)

const (
	HistoryFileName = "run_history.json"
	RunsDir         = "runs"
	MaxHistoryRuns  = 50
)

// ChangedTest represents a test that changed status during a run
type ChangedTest struct {
	Path      string `json:"path"`
	OldStatus string `json:"oldStatus"`
	NewStatus string `json:"newStatus"`
}

// TestEntry represents a test with its status
type TestEntry struct {
	Path   string `json:"path"`
	Status string `json:"status"`
}

// DiffResult contains the computed diff between before and after states
type DiffResult struct {
	Gained  []TestEntry `json:"gained"`
	Lost    []TestEntry `json:"lost"`
	Changed []TestEntry `json:"changed"`
}

// RunOptions contains the original request options for a run
type RunOptions struct {
	Paths      []string `json:"paths,omitempty"`
	Dir        string   `json:"dir,omitempty"`
	FailedOnly bool     `json:"failedOnly,omitempty"`
	Rebuild    bool     `json:"rebuild,omitempty"`
}

// RunHistoryEntry is the summary stored in run_history.json
type RunHistoryEntry struct {
	ID           string        `json:"id"`
	Path         string        `json:"path"`
	Paths        []string      `json:"paths,omitempty"` // Specific test paths (if used)
	Profile      string        `json:"profile,omitempty"`
	Source       string        `json:"source,omitempty"` // "http", "mcp", "stream"
	StartedAt    time.Time     `json:"startedAt"`
	CompletedAt  time.Time     `json:"completedAt,omitempty"`
	Phase        string        `json:"phase"`
	Total        int           `json:"total"`
	Passed       int           `json:"passed"`
	Failed       int           `json:"failed"`
	Skipped      int           `json:"skipped"`
	Crashed      int           `json:"crashed"`
	Timeout      int           `json:"timeout"`
	Gained       int           `json:"gained"`
	Lost         int           `json:"lost"`
	FailedOnly   bool          `json:"failedOnly,omitempty"`
	Rebuild      bool          `json:"rebuild,omitempty"`
	BaselineRef  string        `json:"baselineRef,omitempty"`
	ChangedTests []ChangedTest `json:"changedTests,omitempty"`
	BuildOutput  []string      `json:"buildOutput,omitempty"`
}

// RunDetails contains the full before/after/diff data stored in runs/<id>.json
type RunDetails struct {
	ID       string      `json:"id"`
	Before   []TestEntry `json:"before"`
	After    []TestEntry `json:"after"`
	Diff     DiffResult  `json:"diff"`
	Duration float64     `json:"duration"` // milliseconds
	Status   string      `json:"status"`   // "complete", "cancelled", "error"
	Options  RunOptions  `json:"options"`
}

// RunHistory is the container for all run entries
type RunHistory struct {
	Runs []RunHistoryEntry `json:"runs"`
}

var (
	historyMu   sync.RWMutex
	historyPath string
	runsPath    string
)

func init() {
	historyPath = HistoryFileName
	runsPath = RunsDir
}

// GenerateRunID creates a unique run ID
func GenerateRunID() string {
	bytes := make([]byte, 8)
	rand.Read(bytes)
	return fmt.Sprintf("run-%d-%s", time.Now().Unix(), hex.EncodeToString(bytes))
}

// ensureRunsDir creates the runs directory if it doesn't exist
func ensureRunsDir() error {
	return os.MkdirAll(runsPath, 0755)
}

// LoadHistory loads the run history from disk
func LoadHistory() (*RunHistory, error) {
	historyMu.RLock()
	defer historyMu.RUnlock()

	data, err := os.ReadFile(historyPath)
	if err != nil {
		if os.IsNotExist(err) {
			return &RunHistory{Runs: []RunHistoryEntry{}}, nil
		}
		return nil, err
	}

	var history RunHistory
	if err := json.Unmarshal(data, &history); err != nil {
		return nil, err
	}

	return &history, nil
}

// saveHistoryLocked saves the history (must be called with historyMu held)
func saveHistoryLocked(history *RunHistory) error {
	// Sort by start time (newest first)
	sort.Slice(history.Runs, func(i, j int) bool {
		return history.Runs[i].StartedAt.After(history.Runs[j].StartedAt)
	})

	// Collect IDs to delete if we exceed the limit
	var idsToDelete []string
	if len(history.Runs) > MaxHistoryRuns {
		for _, run := range history.Runs[MaxHistoryRuns:] {
			idsToDelete = append(idsToDelete, run.ID)
		}
		history.Runs = history.Runs[:MaxHistoryRuns]
	}

	data, err := json.MarshalIndent(history, "", "  ")
	if err != nil {
		return err
	}

	if err := os.WriteFile(historyPath, data, 0644); err != nil {
		return err
	}

	// Clean up old detail files
	for _, id := range idsToDelete {
		detailPath := filepath.Join(runsPath, id+".json")
		os.Remove(detailPath) // Ignore errors - file might not exist
	}

	return nil
}

// SaveRun saves both the history entry and the run details
func SaveRun(entry RunHistoryEntry, details *RunDetails) error {
	historyMu.Lock()
	defer historyMu.Unlock()

	// Load existing history
	data, err := os.ReadFile(historyPath)
	var history RunHistory
	if err != nil {
		if !os.IsNotExist(err) {
			return err
		}
		history = RunHistory{Runs: []RunHistoryEntry{}}
	} else {
		if err := json.Unmarshal(data, &history); err != nil {
			return err
		}
	}

	// Add new entry at the beginning
	history.Runs = append([]RunHistoryEntry{entry}, history.Runs...)

	// Save history
	if err := saveHistoryLocked(&history); err != nil {
		return err
	}

	// Save details if provided
	if details != nil {
		if err := ensureRunsDir(); err != nil {
			return err
		}

		detailPath := filepath.Join(runsPath, entry.ID+".json")
		detailData, err := json.MarshalIndent(details, "", "  ")
		if err != nil {
			return err
		}

		if err := os.WriteFile(detailPath, detailData, 0644); err != nil {
			return err
		}
	}

	return nil
}

// AddRunToHistory adds a run entry without details (for backward compatibility)
func AddRunToHistory(entry RunHistoryEntry) error {
	return SaveRun(entry, nil)
}

// LoadRunDetails loads the full details for a specific run
func LoadRunDetails(id string) (*RunDetails, error) {
	detailPath := filepath.Join(runsPath, id+".json")

	data, err := os.ReadFile(detailPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("run details not found: %s", id)
		}
		return nil, err
	}

	var details RunDetails
	if err := json.Unmarshal(data, &details); err != nil {
		return nil, err
	}

	return &details, nil
}

// GetRunEntry finds a specific run entry by ID
func GetRunEntry(id string) (*RunHistoryEntry, error) {
	history, err := LoadHistory()
	if err != nil {
		return nil, err
	}

	for _, run := range history.Runs {
		if run.ID == id {
			return &run, nil
		}
	}

	return nil, fmt.Errorf("run not found: %s", id)
}

// DeleteRun removes a run from history and deletes its detail file
func DeleteRun(id string) error {
	historyMu.Lock()
	defer historyMu.Unlock()

	// Load existing history
	data, err := os.ReadFile(historyPath)
	if err != nil {
		return err
	}

	var history RunHistory
	if err := json.Unmarshal(data, &history); err != nil {
		return err
	}

	// Find and remove the entry
	found := false
	newRuns := make([]RunHistoryEntry, 0, len(history.Runs))
	for _, run := range history.Runs {
		if run.ID != id {
			newRuns = append(newRuns, run)
		} else {
			found = true
		}
	}

	if !found {
		return fmt.Errorf("run not found: %s", id)
	}

	history.Runs = newRuns
	if err := saveHistoryLocked(&history); err != nil {
		return err
	}

	// Delete detail file (ignore errors - might not exist)
	detailPath := filepath.Join(runsPath, id+".json")
	os.Remove(detailPath)

	return nil
}

// ClearHistory removes all history entries and detail files
func ClearHistory() error {
	historyMu.Lock()
	defer historyMu.Unlock()

	// Load history to get IDs for cleanup
	data, err := os.ReadFile(historyPath)
	if err == nil {
		var history RunHistory
		if json.Unmarshal(data, &history) == nil {
			// Delete all detail files
			for _, run := range history.Runs {
				detailPath := filepath.Join(runsPath, run.ID+".json")
				os.Remove(detailPath)
			}
		}
	}

	// Save empty history
	history := &RunHistory{Runs: []RunHistoryEntry{}}
	return saveHistoryLocked(history)
}

// ComputeDiff calculates the diff between before and after test entries
func ComputeDiff(before, after []TestEntry) DiffResult {
	beforeMap := make(map[string]string)
	for _, e := range before {
		beforeMap[e.Path] = e.Status
	}

	afterMap := make(map[string]string)
	for _, e := range after {
		afterMap[e.Path] = e.Status
	}

	// Initialize as empty slices (not nil) for proper JSON serialization
	gained := []TestEntry{}
	lost := []TestEntry{}
	changed := []TestEntry{}

	for _, e := range after {
		beforeStatus, existed := beforeMap[e.Path]
		if !existed {
			continue
		}

		if beforeStatus != e.Status {
			changed = append(changed, e)

			if e.Status == "PASS" && beforeStatus != "PASS" {
				gained = append(gained, e)
			} else if e.Status != "PASS" && beforeStatus == "PASS" {
				lost = append(lost, e)
			}
		}
	}

	return DiffResult{
		Gained:  gained,
		Lost:    lost,
		Changed: changed,
	}
}

// ComputeChangedTests converts a diff to ChangedTest format for history summary
func ComputeChangedTests(before, after []TestEntry) []ChangedTest {
	beforeMap := make(map[string]string)
	for _, e := range before {
		beforeMap[e.Path] = e.Status
	}

	var changedTests []ChangedTest
	for _, e := range after {
		beforeStatus, existed := beforeMap[e.Path]
		if existed && beforeStatus != e.Status {
			changedTests = append(changedTests, ChangedTest{
				Path:      e.Path,
				OldStatus: beforeStatus,
				NewStatus: e.Status,
			})
		}
	}

	return changedTests
}
