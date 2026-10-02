// Groups test results by root cause (panic location + message), ignoring
// per-run noise like thread ids, so identical failures are counted together.
//
//	go run ./runner/cmd/notimpl [-status NOT_IMPLEMENTED] [-examples 3] [-code=false] [-lines N] [-above N] [-below N] [-width N] [results.json]
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
	"unicode/utf8"
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
		hlKeyword, hlString, hlComment, hlNumber, hlMacro, hlType = "", "", "", "", "", ""
	}
}

var ansiRe = regexp.MustCompile(`\x1b\[[0-9;]*m`)

func visibleWidth(s string) int { return utf8.RuneCountInString(ansiRe.ReplaceAllString(s, "")) }

// printBlock prints a cause block, with the source around loc to its right (at column leftW) when the terminal is wide enough.
// codeOpts controls the source window: lines is the total height (0 = match the block),
// above/below override either side of the panic line (-1 = split lines evenly).
type codeOpts struct {
	show                bool
	lines, above, below int
}

func printBlock(block []string, loc string, leftW int, opts codeOpts) {
	var code []string
	if termW, _, err := term.GetSize(int(os.Stdout.Fd())); err == nil && opts.show && loc != "" {
		if w := termW - leftW - 4; w >= 40 {
			height := opts.lines
			if height <= 0 {
				height = max(len(block), 5)
			}
			above, below := opts.above, opts.below
			if above < 0 {
				above = (height - 1 - max(below, 0)) / 2
				if below >= 0 {
					above = max(height-1-below, 0)
				}
			}
			if below < 0 {
				below = max(height-1-above, 0)
			}
			code = snippet(loc, above, below, w)
		}
	}
	for i := range max(len(block), len(code)) {
		l := ""
		if i < len(block) {
			l = block[i]
		}
		if i < len(code) {
			l += strings.Repeat(" ", leftW-visibleWidth(l)+4) + code[i]
		}
		fmt.Println(l)
	}
}

func main() {
	statusName := flag.String("status", "NOT_IMPLEMENTED", "Result status to analyze")
	examples := flag.Int("examples", 3, "Example test paths to show per root cause")
	width := flag.Int("width", 0, "Max width of the cause column next to snippets (0 = auto from terminal width)")
	var code codeOpts
	flag.BoolVar(&code.show, "code", true, "Show source around panic locations (needs a wide terminal)")
	flag.IntVar(&code.lines, "lines", 0, "Total source lines to show (0 = match the cause block height)")
	flag.IntVar(&code.above, "above", -1, "Source lines above the panic line (-1 = derive from -lines)")
	flag.IntVar(&code.below, "below", -1, "Source lines below the panic line (-1 = derive from -lines)")
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
	// cap the cause column of blocks with a snippet so one long path doesn't push all snippets away
	maxW := *width
	if termW, _, err := term.GetSize(int(os.Stdout.Fd())); maxW == 0 && code.show && err == nil {
		maxW = max(60, min(110, termW-64))
	}

	blocks := make([][]string, len(causes))
	for ci, c := range causes {
		// fit shortens s to the room left after indent, cutting the start (paths) or the end (messages)
		fit := func(s string, indent int, keepEnd bool) string {
			rs := []rune(s)
			if n := maxW - indent; c.loc != "" && maxW > 0 && len(rs) > n {
				if keepEnd {
					return "…" + string(rs[len(rs)-n+1:])
				}
				return string(rs[:n-1]) + "…"
			}
			return s
		}

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

		block := []string{fmt.Sprintf("%s%6d%s %s%5.1f%% %s%5.2f%%%s  ", bold+heatColor(float64(len(paths)), 10, 50, 100), len(paths), reset,
			heatColor(pct(len(paths), total), 2, 10, 25), pct(len(paths), total),
			heatColor(pct(len(paths), len(rs)), 0.05, 0.2, 0.5), pct(len(paths), len(rs)), reset)}
		if c.loc != "" {
			block[0] += cyan + fit(c.loc, 22, true) + reset
			block = append(block, fmt.Sprintf("%22s%s%s%s", "", bold, fit(lines[0], 22, false), reset))
		} else {
			block[0] += bold + lines[0] + reset
		}
		for _, l := range lines[1:] {
			block = append(block, fmt.Sprintf("%24s%s", "", fit(l, 24, false)))
		}

		shown := min(*examples, len(paths))
		for _, p := range paths[:shown] {
			block = append(block, fmt.Sprintf("%24s%s· %s%s", "", dim, fit(strings.TrimPrefix(p, "../../test262/test/"), 26, true), reset))
		}
		if rest := len(paths) - shown; rest > 0 && shown > 0 {
			block = append(block, fmt.Sprintf("%24s%s… %d more%s", "", dim, rest, reset))
		}

		blocks[ci] = block
	}

	// align all snippets on one column, past the widest block that has one
	leftW := 0
	for i, c := range causes {
		for _, l := range blocks[i] {
			if c.loc != "" {
				leftW = max(leftW, visibleWidth(l))
			}
		}
	}
	termW, _, termErr := term.GetSize(int(os.Stdout.Fd()))
	for i, c := range causes {
		if i > 0 && code.show && termErr == nil && termW-leftW-4 >= 40 {
			fmt.Printf("%*s%s─────────%s\n", leftW+5, "", dim, reset)
		} else {
			fmt.Println()
		}
		printBlock(blocks[i], c.loc, leftW, code)
	}
}
