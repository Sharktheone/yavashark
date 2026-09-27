package scheduler

import (
	"embed"
	"encoding/json"
	"errors"
	"hash/fnv"
	"io/fs"
	"math"
	"sort"
	"strings"
	"yavashark_test262_runner/calibration"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"
)

// The empty default keeps this pattern valid when the optional costs.json is absent.
//
//go:embed costs*.json
var seedFiles embed.FS
var seeds map[string]string

func init() {
	seedJSON, err := seedFiles.ReadFile("costs.json")
	if errors.Is(err, fs.ErrNotExist) {
		seedJSON, err = seedFiles.ReadFile("costs.default.json")
	}
	if err != nil {
		panic(err)
	}
	if err := json.Unmarshal(seedJSON, &seeds); err != nil {
		panic(err)
	}
}

type Job struct {
	Path     string
	Tier     calibration.Tier
	Cost     float64
	TimedOut bool
}
type Plan struct {
	Fast, Medium, Slow, Risky []Job
	Discovered                int `json:"discovered"`
	QuickEligible             int `json:"quick_eligible"`
	Excluded                  int `json:"excluded"`
	Unknown                   int `json:"unknown"`
}

func Classify(path string, e *calibration.Environment, h map[string]results.Result) Job {
	key := calibration.Key(path)
	j := Job{Path: path, Tier: calibration.Unknown}
	// A known timeout in either source must never enter quick selection.
	if r, ok := h[key]; ok && r.Status == status.TIMEOUT {
		j.Tier = calibration.Risky
		j.TimedOut = true
		return j
	}
	// Measurements for this hardware supersede the conservative checked-in seed.
	if m := e.Tests[key]; m != nil {
		if m.LastStatus == status.TIMEOUT {
			j.Tier = calibration.Risky
			j.TimedOut = true
			return j
		}
		if m.LastStatus == status.CRASH {
			j.Tier = calibration.Risky
			return j
		}
		if m.Samples > 0 && m.CPU > 0 {
			j.Cost = m.CPU
			j.Tier = calibration.CostTier(m.CPU)
			return j
		}
	}
	if r, ok := h[key]; ok {
		if r.Status == status.TIMEOUT {
			j.Tier = calibration.Risky
			j.TimedOut = true
			return j
		}
		if r.Status == status.CRASH {
			j.Tier = calibration.Risky
			return j
		}
		if r.CPUTime > 0 {
			j.Cost = float64(r.CPUTime)
			j.Tier = calibration.CostTier(j.Cost)
			return j
		}
	}
	// These tests construct essentially all Unicode code points even from tiny sources.
	if strings.HasPrefix(key, "built-ins/RegExp/property-escapes/generated/") {
		j.Tier = calibration.Slow
		return j
	}
	if tier, ok := seeds[key]; ok {
		if tier == "timeout" {
			j.Tier = calibration.Risky
			j.TimedOut = true
		} else {
			j.Tier = calibration.Tier(tier)
		}
		return j
	}
	// Known completed tests with no measured CPU cost are normal, not assigned fake times.
	if r, ok := h[key]; ok && r.Status != status.SKIP && r.Status != status.RUNNER_ERROR {
		j.Tier = calibration.Fast
	}
	return j
}
func hash(path string) uint64 {
	h := fnv.New64a()
	h.Write([]byte(calibration.Key(path)))
	return h.Sum64()
}
func Make(paths []string, e *calibration.Environment, h map[string]results.Result, selection string, coverage float64) Plan {
	p := Plan{Discovered: len(paths)}
	var eligible, others []Job
	for _, path := range paths {
		j := Classify(path, e, h)
		if j.Tier == calibration.Unknown {
			p.Unknown++
		}
		if j.Tier == calibration.Fast || j.Tier == calibration.Medium || j.Tier == calibration.Unknown {
			eligible = append(eligible, j)
		} else {
			others = append(others, j)
		}
	}
	p.QuickEligible = len(eligible)
	// Stable, nested sampling across test families. Coverage applies only to this range.
	sort.Slice(eligible, func(i, j int) bool {
		a, b := hash(eligible[i].Path), hash(eligible[j].Path)
		if a == b {
			return eligible[i].Path < eligible[j].Path
		}
		return a < b
	})
	if selection == "quick" {
		n := int(math.Ceil(float64(len(eligible)) * coverage / 100))
		eligible = eligible[:n]
	} else {
		eligible = append(eligible, others...)
	}
	for _, j := range eligible {
		switch j.Tier {
		case calibration.Fast:
			p.Fast = append(p.Fast, j)
		case calibration.Slow:
			p.Slow = append(p.Slow, j)
		case calibration.Risky:
			p.Risky = append(p.Risky, j)
		default:
			p.Medium = append(p.Medium, j)
		}
	}
	p.Excluded = p.Discovered - len(eligible)
	// Longest measured jobs first within a phase reduces its completion tail.
	for _, jobs := range [][]Job{p.Fast, p.Medium, p.Slow, p.Risky} {
		sort.SliceStable(jobs, func(i, j int) bool {
			if jobs[i].TimedOut != jobs[j].TimedOut {
				return jobs[i].TimedOut
			} // launch hang timers early
			if jobs[i].Cost != jobs[j].Cost {
				return jobs[i].Cost > jobs[j].Cost
			}
			return hash(jobs[i].Path) < hash(jobs[j].Path)
		})
	}
	return p
}
