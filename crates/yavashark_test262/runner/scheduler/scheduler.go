package scheduler

import (
	"encoding/json"
	"log"
	"os"
	"sort"
	"time"
)

type TestJob struct {
	Path          string
	EstimatedTime time.Duration
	Priority      int
}

type StoredResult struct {
	Status   string        `json:"status"`
	Msg      string        `json:"msg"`
	Path     string        `json:"path"`
	MemoryKB uint64        `json:"memory_kb"`
	Duration time.Duration `json:"duration"`
}

const (
	// Priority levels
	PRIORITY_FAST      = 0 // Tests < 100ms based on history
	PRIORITY_MEDIUM    = 1 // Tests 100ms - 1s
	PRIORITY_SLOW      = 2 // Tests 1s - 5s
	PRIORITY_SLOW_RISK = 3 // Tests > 5s or timeout/crash
)

// ScheduleTests sorts tests intelligently using historical timing data
func ScheduleTests(testPaths []string, timings map[string]time.Duration) []TestJob {
	jobs := make([]TestJob, len(testPaths))

	for i, path := range testPaths {
		estimatedTime := timings[path]
		priority := calculatePriorityFromTiming(estimatedTime)

		jobs[i] = TestJob{
			Path:          path,
			EstimatedTime: estimatedTime,
			Priority:      priority,
		}
	}

	sort.Slice(jobs, func(i, j int) bool {
		if jobs[i].Priority != jobs[j].Priority {
			return jobs[i].Priority < jobs[j].Priority
		}
		if jobs[i].EstimatedTime != jobs[j].EstimatedTime {
			return jobs[i].EstimatedTime < jobs[j].EstimatedTime
		}
		return jobs[i].Path < jobs[j].Path
	})

	return jobs
}

func calculatePriorityFromTiming(duration time.Duration) int {
	if duration == 0 {
		return PRIORITY_MEDIUM
	}

	if duration > 5*time.Second {
		return PRIORITY_SLOW_RISK
	} else if duration > 1*time.Second {
		return PRIORITY_SLOW
	} else if duration > 100*time.Millisecond {
		return PRIORITY_MEDIUM
	}
	return PRIORITY_FAST
}

func LoadTestTimings(resultsPath string) map[string]time.Duration {
	timings := make(map[string]time.Duration)

	contents, err := os.ReadFile(resultsPath)
	if err != nil {
		if os.IsNotExist(err) {
			log.Printf("No previous results found at %s, using default priorities", resultsPath)
			return timings
		}
		log.Printf("Failed to read results file %s: %v", resultsPath, err)
		return timings
	}

	var results []StoredResult
	err = json.Unmarshal(contents, &results)
	if err != nil {
		log.Printf("Failed to parse results JSON: %v", err)
		return timings
	}

	for _, result := range results {
		path := result.Path

		if result.Status == "TIMEOUT" {
			timings[path] = 30 * time.Second
		} else if result.Status == "CRASH" {
			timings[path] = 20 * time.Second
		} else if result.Duration > 0 {
			timings[path] = result.Duration
		} else {
			timings[path] = 500 * time.Millisecond
		}
	}

	log.Printf("Loaded timing data for %d tests from %s", len(timings), resultsPath)
	return timings
}

func EstimateTimingFromFileSize(path string) time.Duration {
	fileInfo, err := os.Stat(path)
	if err != nil {
		return 500 * time.Millisecond
	}

	sizeKB := fileInfo.Size() / 1024

	if sizeKB > 100 {
		return 10 * time.Second
	} else if sizeKB > 50 {
		return 5 * time.Second
	} else if sizeKB > 20 {
		return 1 * time.Second
	} else if sizeKB > 5 {
		return 200 * time.Millisecond
	}
	return 50 * time.Millisecond
}

func EnrichTimingsWithFallback(timings map[string]time.Duration, testPaths []string) {
	for _, path := range testPaths {
		if _, exists := timings[path]; !exists {
			timings[path] = EstimateTimingFromFileSize(path)
		}
	}
}

func GetStatistics(timings map[string]time.Duration) (min, max, avg time.Duration, fastCount, mediumCount, slowCount, riskCount int) {
	if len(timings) == 0 {
		return
	}

	var total time.Duration

	for _, duration := range timings {
		total += duration

		priority := calculatePriorityFromTiming(duration)
		switch priority {
		case PRIORITY_FAST:
			fastCount++
		case PRIORITY_MEDIUM:
			mediumCount++
		case PRIORITY_SLOW:
			slowCount++
		case PRIORITY_SLOW_RISK:
			riskCount++
		}

		if min == 0 || duration < min {
			min = duration
		}
		if duration > max {
			max = duration
		}
	}

	avg = total / time.Duration(len(timings))
	return
}
