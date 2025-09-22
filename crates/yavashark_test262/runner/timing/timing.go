package timing

import (
	"strings"
	"sync"
	"time"
)

var (
	Mu                    sync.Mutex
	TotalDuration         time.Duration
	RealmCreationDuration time.Duration
	ParsingDuration       time.Duration
	SetupDuration         time.Duration
)

func ParseDurations(buf string) {
	lines := strings.Lines(buf)

	var total time.Duration
	var realm time.Duration
	var parse time.Duration
	var setup time.Duration

	for line := range lines {
		if strings.HasPrefix(line, "TOTAL: ") {
			ns := strings.TrimPrefix(line, "TOTAL: ")
			ns = strings.TrimSpace(ns)
			total, _ = time.ParseDuration(ns + "ns")
		}

		if strings.HasPrefix(line, "REALM: ") {
			ns := strings.TrimPrefix(line, "REALM: ")
			ns = strings.TrimSpace(ns)
			realm, _ = time.ParseDuration(ns + "ns")
		}

		if strings.HasPrefix(line, "PARSE: ") {
			ns := strings.TrimPrefix(line, "PARSE: ")
			ns = strings.TrimSpace(ns)
			parse, _ = time.ParseDuration(ns + "ns")
		}

		if strings.HasPrefix(line, "SETUP: ") {
			ns := strings.TrimPrefix(line, "SETUP: ")
			ns = strings.TrimSpace(ns)
			setup, _ = time.ParseDuration(ns + "ns")
		}
	}

	go func() {
		Mu.Lock()
		defer Mu.Unlock()

		TotalDuration += total
		RealmCreationDuration += realm
		ParsingDuration += parse
		SetupDuration += setup
	}()
}

func PrintTimings() {
	println("=== Timings ===")
	println("Total time:         ", TotalDuration.String())
	println("Realm creation:     ", RealmCreationDuration.String())
	println("Parsing:            ", ParsingDuration.String())
	println("Setup:              ", SetupDuration.String())
	println("================")

	println("=== Percentages ===")
	if TotalDuration > 0 {
		println("Realm creation:     ", float64(RealmCreationDuration)/float64(TotalDuration)*100, "%")
		println("Parsing:            ", float64(ParsingDuration)/float64(TotalDuration)*100, "%")
		println("Setup:              ", float64(SetupDuration)/float64(TotalDuration)*100, "%")

		if SetupDuration > 0 {
			println("Realm creation in setup: ", float64(RealmCreationDuration)/float64(SetupDuration)*100, "%")
		}

	}

}
