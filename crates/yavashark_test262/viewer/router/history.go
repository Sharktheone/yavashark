package router

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"

	"github.com/gofiber/fiber/v2"
)

const (
	historyFileName = "run_history.json"
	maxHistoryRuns  = 50
)

type ChangedTest struct {
	Path      string `json:"path"`
	OldStatus string `json:"oldStatus"`
	NewStatus string `json:"newStatus"`
}

type RunHistoryEntry struct {
	ID           string        `json:"id"`
	Path         string        `json:"path"`
	Profile      string        `json:"profile,omitempty"`
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
	BaselineRef  string        `json:"baselineRef,omitempty"`
	ChangedTests []ChangedTest `json:"changedTests,omitempty"`
	BuildOutput  []string      `json:"buildOutput,omitempty"`
}

type RunHistory struct {
	Runs []RunHistoryEntry `json:"runs"`
}

var (
	historyMu   sync.RWMutex
	historyPath string
)

func init() {
	historyPath = filepath.Join("run_history.json")
}

func loadHistory() (*RunHistory, error) {
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

func saveHistory(history *RunHistory) error {
	historyMu.Lock()
	defer historyMu.Unlock()

	sort.Slice(history.Runs, func(i, j int) bool {
		return history.Runs[i].StartedAt.After(history.Runs[j].StartedAt)
	})

	if len(history.Runs) > maxHistoryRuns {
		history.Runs = history.Runs[:maxHistoryRuns]
	}

	data, err := json.MarshalIndent(history, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(historyPath, data, 0644)
}

func AddRunToHistory(entry RunHistoryEntry) error {
	history, err := loadHistory()
	if err != nil {
		history = &RunHistory{Runs: []RunHistoryEntry{}}
	}

	history.Runs = append([]RunHistoryEntry{entry}, history.Runs...)
	return saveHistory(history)
}

func getRunHistory(c *fiber.Ctx) error {
	history, err := loadHistory()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to load history: " + err.Error(),
		})
	}

	return c.JSON(history)
}

func getRunHistoryEntry(c *fiber.Ctx) error {
	id := c.Params("id")
	if id == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Run ID is required",
		})
	}

	history, err := loadHistory()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to load history: " + err.Error(),
		})
	}

	for _, run := range history.Runs {
		if run.ID == id {
			return c.JSON(run)
		}
	}

	return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
		"error": "Run not found",
	})
}

func deleteRunHistoryEntry(c *fiber.Ctx) error {
	id := c.Params("id")
	if id == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Run ID is required",
		})
	}

	history, err := loadHistory()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to load history: " + err.Error(),
		})
	}

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
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "Run not found",
		})
	}

	history.Runs = newRuns
	if err := saveHistory(history); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to save history: " + err.Error(),
		})
	}

	return c.JSON(fiber.Map{
		"status": "deleted",
		"id":     id,
	})
}

func clearRunHistory(c *fiber.Ctx) error {
	history := &RunHistory{Runs: []RunHistoryEntry{}}
	if err := saveHistory(history); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to clear history: " + err.Error(),
		})
	}

	return c.JSON(fiber.Map{
		"status": "cleared",
	})
}
