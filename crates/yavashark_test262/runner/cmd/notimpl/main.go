// Groups test results by root cause (panic location + message), ignoring
// per-run noise like thread ids, so identical failures are counted together.
//
//	go run ./runner/cmd/notimpl [-status NOT_IMPLEMENTED] [-examples 3] [results.json]
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"regexp"
	"sort"
	"strings"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/status"

	"golang.org/x/term"
)

var (
	panicRe  = regexp.MustCompile(`(?s)panicked at (\S+):\n(.*?)(?:\nnote: run with|$)`)
	threadRe = regexp.MustCompile(`thread '([^']*)' \(\d+\)`)
)

type cause struct{ loc, msg string }

func rootCause(msg string) cause {
	if m := panicRe.FindStringSubmatch(msg); m != nil {
		return cause{m[1], strings.TrimSpace(m[2])}
	}
	return cause{"", threadRe.ReplaceAllString(strings.TrimSpace(msg), "thread '$1'")}
}

// ANSI styles, disabled when stdout isn't a terminal or NO_COLOR is set.
var bold, dim, pink, cyan, reset = "\x1b[1m", "\x1b[2m", "\x1b[38;5;212m", "\x1b[38;5;117m", "\x1b[0m"

// Muted heat scale for percentages: sage, khaki, clay, rose.
var heat = []string{"\x1b[38;5;108m", "\x1b[38;5;144m", "\x1b[38;5;173m", "\x1b[38;5;167m"}

// heatColor picks a heat color for v given ascending thresholds.
func heatColor(v float64, steps ...float64) string {
	i := 0
	for i < len(steps) && v >= steps[i] {
		i++
	}
	return heat[i]
}

func init() {
	if !term.IsTerminal(int(os.Stdout.Fd())) || os.Getenv("NO_COLOR") != "" {
		bold, dim, pink, cyan, reset = "", "", "", "", ""
		heat = []string{"", "", "", ""}
	}
}

func main() {
	statusName := flag.String("status", "NOT_IMPLEMENTED", "Result status to analyze")
	examples := flag.Int("examples", 3, "Example test paths to show per root cause")
	flag.Parse()

	want, err := status.ParseStatus(*statusName)
	if err != nil {
		log.Fatal(err)
	}
	path := results.RESULT_PATH
	if flag.NArg() > 0 {
		path = flag.Arg(0)
	}
	data, err := os.ReadFile(path)
	if err != nil {
		log.Fatal(err)
	}
	var rs []results.Result
	if err := json.Unmarshal(data, &rs); err != nil {
		log.Fatal(err)
	}

	groups := map[cause][]string{}
	total := 0
	for _, r := range rs {
		if r.Status == want {
			c := rootCause(r.Msg)
			groups[c] = append(groups[c], r.Path)
			total++
		}
	}

	causes := make([]cause, 0, len(groups))
	for c := range groups {
		causes = append(causes, c)
	}
	sort.Slice(causes, func(i, j int) bool {
		a, b := len(groups[causes[i]]), len(groups[causes[j]])
		if a != b {
			return a > b
		}
		return causes[i].loc+causes[i].msg < causes[j].loc+causes[j].msg
	})

	pct := func(n, of int) float64 { return 100 * float64(n) / float64(of) }
	fmt.Printf("%s%s%d %s%s%s results %s(%s%.2f%%%s of %d tests) · %d root causes%s\n", bold, heatColor(float64(total), 100, 1000, 5000), total, pink, want, reset+bold, reset+dim, heatColor(pct(total, len(rs)), 1, 5, 15), pct(total, len(rs)), reset+dim, len(rs), len(causes), reset)
	for _, c := range causes {
		paths := groups[c]
		sort.Strings(paths)

		var lines []string
		for _, l := range strings.Split(c.msg, "\n") {
			if l = strings.TrimRight(l, " \t"); strings.TrimSpace(l) != "" {
				lines = append(lines, l)
			}
		}
		if len(lines) == 0 {
			lines = []string{"(no message)"}
		}

		fmt.Printf("\n%s%6d%s %s%5.1f%% %s%5.2f%%%s  ", bold+heatColor(float64(len(paths)), 10, 50, 100), len(paths), reset,
			heatColor(pct(len(paths), total), 2, 10, 25), pct(len(paths), total),
			heatColor(pct(len(paths), len(rs)), 0.05, 0.2, 0.5), pct(len(paths), len(rs)), reset)
		if c.loc != "" {
			fmt.Printf("%s%s%s\n%22s", cyan, c.loc, reset, "")
		}
		fmt.Printf("%s%s%s\n", bold, lines[0], reset)
		for _, l := range lines[1:] {
			fmt.Printf("%24s%s\n", "", l)
		}

		shown := min(*examples, len(paths))
		for _, p := range paths[:shown] {
			fmt.Printf("%24s%s· %s%s\n", "", dim, strings.TrimPrefix(p, "../../test262/test/"), reset)
		}
		if rest := len(paths) - shown; rest > 0 && shown > 0 {
			fmt.Printf("%24s%s… %d more%s\n", "", dim, rest, reset)
		}
	}
}
