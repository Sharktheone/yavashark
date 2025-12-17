package progress

import (
	"fmt"
	"math"
	"os"
	"strings"
	"sync"
	"sync/atomic"
	"yavashark_test262_runner/status"

	"golang.org/x/term"
)

const (
	ColorReset = "\033[0m"
	ColorBold  = "\033[1m"

	// Foreground colors
	FgGray        = "\033[90m"
	FgBrightGreen = "\033[92m"
	FgBrightRed   = "\033[91m"
)

const (
	BlockFull = "█"
	Block7_8  = "▉"
	Block6_8  = "▊"
	Block5_8  = "▋"
	Block4_8  = "▌"
	Block3_8  = "▍"
	Block2_8  = "▎"
	Block1_8  = "▏"
)

// Unicode arrows
const (
	ArrowUp    = "↑"
	ArrowDown  = "↓"
	ArrowRight = "→"
)

// RGB color type
type RGB struct {
	R, G, B int
}

type HSV struct {
	H, S, V float64
}

func rgbToHSV(c RGB) HSV {
	r := float64(c.R) / 255.0
	g := float64(c.G) / 255.0
	b := float64(c.B) / 255.0

	max := math.Max(r, math.Max(g, b))
	min := math.Min(r, math.Min(g, b))
	delta := max - min

	var h float64
	s := 0.0
	v := max

	if max > 0 {
		s = delta / max
	}

	if delta != 0 {
		switch max {
		case r:
			h = (g - b) / delta
			if g < b {
				h += 6
			}
		case g:
			h = (b-r)/delta + 2
		case b:
			h = (r-g)/delta + 4
		}
		h *= 60
	}

	return HSV{H: h, S: s, V: v}
}

func hsvToRGB(hsv HSV) RGB {
	h := hsv.H
	s := hsv.S
	v := hsv.V

	c := v * s
	x := c * (1 - math.Abs(math.Mod(h/60.0, 2)-1))
	m := v - c

	var r1, g1, b1 float64
	switch {
	case h >= 0 && h < 60:
		r1, g1, b1 = c, x, 0
	case h >= 60 && h < 120:
		r1, g1, b1 = x, c, 0
	case h >= 120 && h < 180:
		r1, g1, b1 = 0, c, x
	case h >= 180 && h < 240:
		r1, g1, b1 = 0, x, c
	case h >= 240 && h < 300:
		r1, g1, b1 = x, 0, c
	default:
		r1, g1, b1 = c, 0, x
	}

	return RGB{
		R: int(math.Round((r1 + m) * 255)),
		G: int(math.Round((g1 + m) * 255)),
		B: int(math.Round((b1 + m) * 255)),
	}
}

func clamp(x, lo, hi float64) float64 {
	if x < lo {
		return lo
	}
	if x > hi {
		return hi
	}
	return x
}

func blendHSV(a, b HSV, t float64) HSV {
	t = clamp(t, 0, 1)

	ha := a.H
	hb := b.H
	d := hb - ha
	if d > 180 {
		d -= 360
	} else if d < -180 {
		d += 360
	}

	h := ha + d*t
	if h < 0 {
		h += 360
	} else if h >= 360 {
		h -= 360
	}

	return HSV{
		H: h,
		S: a.S + (b.S-a.S)*t,
		V: a.V + (b.V-a.V)*t,
	}
}

func shadeRGB(c RGB, vScale float64) RGB {
	hsv := rgbToHSV(c)
	hsv.V = clamp(hsv.V*vScale, 0, 1)
	return hsvToRGB(hsv)
}

// Status colors for progress bar
var (
	colorPass    = RGB{76, 175, 80}  // Green
	colorFail    = RGB{244, 67, 54}  // Red
	colorTimeout = RGB{255, 193, 7}  // Yellow/Amber
	colorCrash   = RGB{156, 39, 176} // Purple
	colorSkip    = RGB{0, 188, 212}  // Cyan
	colorOther   = RGB{33, 150, 243} // Blue
	colorEmpty   = RGB{60, 60, 60}   // Dark gray
)

type PillStyle struct {
	Left  RGB
	Right RGB
	Text  RGB
}

var (
	pillStylePass = PillStyle{
		Left:  RGB{67, 160, 71},
		Right: RGB{46, 125, 50},
		Text:  RGB{255, 255, 255},
	}
	pillStyleFail = PillStyle{
		Left:  RGB{229, 57, 53},
		Right: RGB{198, 40, 40},
		Text:  RGB{255, 255, 255},
	}
	pillStyleSkip = PillStyle{
		Left:  RGB{0, 172, 193},
		Right: RGB{0, 131, 143},
		Text:  RGB{255, 255, 255},
	}
	pillStyleTimeout = PillStyle{
		Left:  RGB{255, 179, 0},
		Right: RGB{255, 143, 0},
		Text:  RGB{33, 33, 33},
	}
	pillStyleCrash = PillStyle{
		Left:  RGB{142, 36, 170},
		Right: RGB{106, 27, 154},
		Text:  RGB{255, 255, 255},
	}
	pillStyleParseError = PillStyle{
		Left:  RGB{30, 136, 229},
		Right: RGB{21, 101, 192},
		Text:  RGB{255, 255, 255},
	}
	pillStyleNotImpl = PillStyle{
		Left:  RGB{117, 117, 117},
		Right: RGB{66, 66, 66},
		Text:  RGB{255, 255, 255},
	}
	pillStyleRunnerError = PillStyle{
		Left:  RGB{97, 97, 97},
		Right: RGB{55, 71, 79},
		Text:  RGB{255, 255, 255},
	}
)

type RecentChange struct {
	Path       string
	Status     status.Status
	PrevStatus status.Status
	IsNew      bool
}

type Summary struct {
	Passed            uint32
	Failed            uint32
	Skipped           uint32
	Timeout           uint32
	Crashed           uint32
	ParseError        uint32
	ParseSuccessError uint32
	NotImplemented    uint32
	RunnerError       uint32
	Total             uint32

	PassGained int32
	PassLost   int32
	FailGained int32
	FailLost   int32

	SkipGained     int32
	SkipLost       int32
	TimeoutGained  int32
	TimeoutLost    int32
	CrashGained    int32
	CrashLost      int32
	ParseErrGained int32
	ParseErrLost   int32
	ParseSucGained int32
	ParseSucLost   int32
	NotImplGained  int32
	NotImplLost    int32
	RunErrGained   int32
	RunErrLost     int32
}

type ProgressTracker struct {
	mu sync.Mutex

	passed              atomic.Uint32
	failed              atomic.Uint32
	skipped             atomic.Uint32
	timeout             atomic.Uint32
	crashed             atomic.Uint32
	parseError          atomic.Uint32
	parseSuccessError   atomic.Uint32
	notImplemented      atomic.Uint32
	runnerError         atomic.Uint32
	total               atomic.Uint32
	lastPrintedProgress atomic.Uint32

	skippedGained           atomic.Int32
	skippedLost             atomic.Int32
	timeoutGained           atomic.Int32
	timeoutLost             atomic.Int32
	crashedGained           atomic.Int32
	crashedLost             atomic.Int32
	parseErrorGained        atomic.Int32
	parseErrorLost          atomic.Int32
	parseSuccessErrorGained atomic.Int32
	parseSuccessErrorLost   atomic.Int32
	notImplementedGained    atomic.Int32
	notImplementedLost      atomic.Int32
	runnerErrorGained       atomic.Int32
	runnerErrorLost         atomic.Int32

	totalTests  uint32
	interactive bool

	prevResults map[string]status.Status

	passGained atomic.Int32
	passLost   atomic.Int32
	failGained atomic.Int32
	failLost   atomic.Int32

	gainedByStatus map[status.Status]*atomic.Int32
	lostByStatus   map[status.Status]*atomic.Int32

	recentChanges []RecentChange
	changesMu     sync.Mutex
}

func NewProgressTracker(totalTests uint32, interactive bool, prevResults map[string]status.Status) *ProgressTracker {
	pt := &ProgressTracker{
		totalTests:    totalTests,
		interactive:   interactive,
		prevResults:   prevResults,
		recentChanges: make([]RecentChange, 0),
	}

	pt.gainedByStatus = map[status.Status]*atomic.Int32{
		status.SKIP:                &pt.skippedGained,
		status.TIMEOUT:             &pt.timeoutGained,
		status.CRASH:               &pt.crashedGained,
		status.PARSE_ERROR:         &pt.parseErrorGained,
		status.PARSE_SUCCESS_ERROR: &pt.parseSuccessErrorGained,
		status.NOT_IMPLEMENTED:     &pt.notImplementedGained,
		status.RUNNER_ERROR:        &pt.runnerErrorGained,
	}
	pt.lostByStatus = map[status.Status]*atomic.Int32{
		status.SKIP:                &pt.skippedLost,
		status.TIMEOUT:             &pt.timeoutLost,
		status.CRASH:               &pt.crashedLost,
		status.PARSE_ERROR:         &pt.parseErrorLost,
		status.PARSE_SUCCESS_ERROR: &pt.parseSuccessErrorLost,
		status.NOT_IMPLEMENTED:     &pt.notImplementedLost,
		status.RUNNER_ERROR:        &pt.runnerErrorLost,
	}

	if interactive {
		pt.enterInteractiveMode()
	}

	return pt
}

func (pt *ProgressTracker) enterInteractiveMode() {
	fmt.Print("\033[?1049h") // alternate screen
	fmt.Print("\033[?25l")   // hide cursor
	fmt.Print("\033[?7l")    // disable line wrap
	fmt.Print("\033[2J\033[H")

	pt.renderInteractive(0)
}

func (pt *ProgressTracker) exitInteractiveMode() {
	fmt.Print("\033[?7h")  // enable line wrap
	fmt.Print("\033[?25h") // show cursor
	fmt.Print("\033[?1049l")
}

func (pt *ProgressTracker) Add(s status.Status, path string) {
	switch s {
	case status.PASS:
		pt.passed.Add(1)
	case status.FAIL:
		pt.failed.Add(1)
	case status.SKIP:
		pt.skipped.Add(1)
	case status.TIMEOUT:
		pt.timeout.Add(1)
	case status.CRASH:
		pt.crashed.Add(1)
	case status.PARSE_ERROR:
		pt.parseError.Add(1)
	case status.PARSE_SUCCESS_ERROR:
		pt.parseSuccessError.Add(1)
	case status.NOT_IMPLEMENTED:
		pt.notImplemented.Add(1)
	case status.RUNNER_ERROR:
		pt.runnerError.Add(1)
	}

	if pt.prevResults != nil {
		prevStatus, existed := pt.prevResults[path]

		if existed && prevStatus != s {
			if prevStatus == status.PASS && s != status.PASS {
				pt.passLost.Add(1)
			}
			if prevStatus != status.PASS && s == status.PASS {
				pt.passGained.Add(1)
			}
			if prevStatus == status.FAIL && s != status.FAIL {
				pt.failLost.Add(1)
			}
			if prevStatus != status.FAIL && s == status.FAIL {
				pt.failGained.Add(1)
			}

			if gained, ok := pt.gainedByStatus[s]; ok {
				gained.Add(1)
			}
			if lost, ok := pt.lostByStatus[prevStatus]; ok {
				lost.Add(1)
			}

			if pt.interactive {
				change := RecentChange{
					Path:       path,
					Status:     s,
					PrevStatus: prevStatus,
					IsNew:      !existed,
				}
				pt.changesMu.Lock()
				pt.recentChanges = append(pt.recentChanges, change)
				if len(pt.recentChanges) > 20 {
					pt.recentChanges = pt.recentChanges[len(pt.recentChanges)-20:]
				}
				pt.changesMu.Unlock()
			}
		}
	}

	current := pt.total.Add(1)
	pt.updateProgress(current)
}

func (pt *ProgressTracker) updateProgress(current uint32) {
	lastPrinted := pt.lastPrintedProgress.Load()

	threshold := uint32(100)
	if pt.totalTests > 0 {
		if pt.interactive {
			threshold = 1
		} else {
			percentThreshold := pt.totalTests / 50
			if percentThreshold > threshold {
				threshold = percentThreshold
			}
		}
	}

	if current-lastPrinted >= threshold || current == pt.totalTests {
		if pt.lastPrintedProgress.CompareAndSwap(lastPrinted, current) {
			if pt.interactive {
				pt.renderInteractive(current)
			} else {
				pt.printProgressBar(current)
			}
		}
	}
}

func (pt *ProgressTracker) Finish() Summary {
	if !pt.interactive {
		pt.printProgressBar(pt.total.Load())
		fmt.Println()
	}

	if pt.interactive {
		pt.renderInteractive(pt.total.Load())
		pt.exitInteractiveMode()
	}

	s := pt.Summary()
	return s
}

func (pt *ProgressTracker) Summary() Summary {
	return Summary{
		Passed:            pt.passed.Load(),
		Failed:            pt.failed.Load(),
		Skipped:           pt.skipped.Load(),
		Timeout:           pt.timeout.Load(),
		Crashed:           pt.crashed.Load(),
		ParseError:        pt.parseError.Load(),
		ParseSuccessError: pt.parseSuccessError.Load(),
		NotImplemented:    pt.notImplemented.Load(),
		RunnerError:       pt.runnerError.Load(),
		Total:             pt.total.Load(),
		PassGained:        pt.passGained.Load(),
		PassLost:          pt.passLost.Load(),
		FailGained:        pt.failGained.Load(),
		FailLost:          pt.failLost.Load(),
		SkipGained:        pt.skippedGained.Load(),
		SkipLost:          pt.skippedLost.Load(),
		TimeoutGained:     pt.timeoutGained.Load(),
		TimeoutLost:       pt.timeoutLost.Load(),
		CrashGained:       pt.crashedGained.Load(),
		CrashLost:         pt.crashedLost.Load(),
		ParseErrGained:    pt.parseErrorGained.Load(),
		ParseErrLost:      pt.parseErrorLost.Load(),
		ParseSucGained:    pt.parseSuccessErrorGained.Load(),
		ParseSucLost:      pt.parseSuccessErrorLost.Load(),
		NotImplGained:     pt.notImplementedGained.Load(),
		NotImplLost:       pt.notImplementedLost.Load(),
		RunErrGained:      pt.runnerErrorGained.Load(),
		RunErrLost:        pt.runnerErrorLost.Load(),
	}
}

func (pt *ProgressTracker) getTerminalWidth() int {
	width, _, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil || width < 40 {
		return 80
	}
	return width
}

func (pt *ProgressTracker) getTerminalHeight() int {
	_, height, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil || height < 10 {
		return 24
	}
	return height
}

func formatPill(label string, style PillStyle) string {
	// Rounded Powerline pill:  <label> 
	// Keep it narrow: exactly one space padding.
	bg := fmt.Sprintf("\033[48;2;%d;%d;%dm", style.Left.R, style.Left.G, style.Left.B)
	fg := fmt.Sprintf("\033[38;2;%d;%d;%dm", style.Text.R, style.Text.G, style.Text.B)

	capFg := fmt.Sprintf("\033[38;2;%d;%d;%dm", style.Left.R, style.Left.G, style.Left.B)

	// Only color the glyph foreground; keep glyph background default.
	// Also reset before the right cap to avoid bleeding the pill background.
	return fmt.Sprintf("%s%s%s %s %s%s%s", capFg, bg, fg, label, ColorReset, capFg, ColorReset)
}

func FormatStatusPill(s status.Status) string {
	switch s {
	case status.PASS:
		return formatPill("PASS", pillStylePass)
	case status.FAIL:
		return formatPill("FAIL", pillStyleFail)
	case status.SKIP:
		return formatPill("SKIP", pillStyleSkip)
	case status.TIMEOUT:
		return formatPill("TIMEOUT", pillStyleTimeout)
	case status.CRASH:
		return formatPill("CRASH", pillStyleCrash)
	case status.PARSE_ERROR:
		return formatPill("PARSE_ERR", pillStyleParseError)
	case status.PARSE_SUCCESS_ERROR:
		return formatPill("PARSE_SUC", pillStyleParseError)
	case status.NOT_IMPLEMENTED:
		return formatPill("NOT_IMPL", pillStyleNotImpl)
	case status.RUNNER_ERROR:
		return formatPill("RUN_ERR", pillStyleRunnerError)
	default:
		return formatPill("???", pillStyleRunnerError)
	}
}

func formatDelta(gained, lost int32) string {
	total := gained - lost

	var totalStr string
	if total > 0 {
		totalStr = fmt.Sprintf("%s+%d%s", FgBrightGreen, total, ColorReset)
	} else if total < 0 {
		totalStr = fmt.Sprintf("%s%d%s", FgBrightRed, total, ColorReset)
	} else {
		totalStr = fmt.Sprintf("%s0%s", FgGray, ColorReset)
	}

	var breakdown string
	if gained > 0 || lost > 0 {
		breakdown = fmt.Sprintf(" (%s%s%d%s %s%s%d%s)",
			FgBrightGreen, ArrowUp, gained, ColorReset,
			FgBrightRed, ArrowDown, lost, ColorReset)
	}

	return totalStr + breakdown
}

func (pt *ProgressTracker) renderInteractive(current uint32) {
	width := pt.getTerminalWidth()
	height := pt.getTerminalHeight()

	fmt.Print("\033[H\033[2J")

	passed := pt.passed.Load()
	failed := pt.failed.Load()
	skipped := pt.skipped.Load()
	timeout := pt.timeout.Load()
	crashed := pt.crashed.Load()
	parseError := pt.parseError.Load()
	parseSuccessError := pt.parseSuccessError.Load()
	notImplemented := pt.notImplemented.Load()
	runnerError := pt.runnerError.Load()

	fmt.Printf("%s Test262 Runner %s\n\n", ColorBold, ColorReset)

	pt.renderFullWidthProgressBar(current, width)
	fmt.Println()
	fmt.Println()

	pt.renderStatusLine("PASS", passed, pt.totalTests, pt.passGained.Load(), pt.passLost.Load(), pillStylePass)
	pt.renderStatusLine("FAIL", failed, pt.totalTests, pt.failGained.Load(), pt.failLost.Load(), pillStyleFail)
	pt.renderStatusLine("SKIP", skipped, pt.totalTests, pt.skippedGained.Load(), pt.skippedLost.Load(), pillStyleSkip)
	pt.renderStatusLine("TIMEOUT", timeout, pt.totalTests, pt.timeoutGained.Load(), pt.timeoutLost.Load(), pillStyleTimeout)
	pt.renderStatusLine("CRASH", crashed, pt.totalTests, pt.crashedGained.Load(), pt.crashedLost.Load(), pillStyleCrash)
	pt.renderStatusLine("PARSE_ERR", parseError, pt.totalTests, pt.parseErrorGained.Load(), pt.parseErrorLost.Load(), pillStyleParseError)
	pt.renderStatusLine("PARSE_SUC", parseSuccessError, pt.totalTests, pt.parseSuccessErrorGained.Load(), pt.parseSuccessErrorLost.Load(), pillStyleParseError)
	pt.renderStatusLine("NOT_IMPL", notImplemented, pt.totalTests, pt.notImplementedGained.Load(), pt.notImplementedLost.Load(), pillStyleNotImpl)
	pt.renderStatusLine("RUN_ERR", runnerError, pt.totalTests, pt.runnerErrorGained.Load(), pt.runnerErrorLost.Load(), pillStyleRunnerError)

	fmt.Println()

	pt.changesMu.Lock()
	changes := make([]RecentChange, len(pt.recentChanges))
	copy(changes, pt.recentChanges)
	pt.changesMu.Unlock()

	headerLines := 13
	availableLines := height - headerLines - 2
	if availableLines < 3 {
		availableLines = 3
	}

	fmt.Printf("%s Recent Changes: %s\n", ColorBold, ColorReset)

	startIdx := 0
	if len(changes) > availableLines {
		startIdx = len(changes) - availableLines
	}

	for i := startIdx; i < len(changes); i++ {
		change := changes[i]
		shortPath := change.Path
		maxPathLen := width - 60
		if maxPathLen < 20 {
			maxPathLen = 20
		}
		if len(shortPath) > maxPathLen {
			shortPath = "..." + shortPath[len(shortPath)-(maxPathLen-3):]
		}

		arrow := ArrowRight
		if change.Status == status.PASS && change.PrevStatus != status.PASS {
			arrow = fmt.Sprintf("%s%s%s", FgBrightGreen, ArrowUp, ColorReset)
		} else if change.PrevStatus == status.PASS && change.Status != status.PASS {
			arrow = fmt.Sprintf("%s%s%s", FgBrightRed, ArrowDown, ColorReset)
		}

		fmt.Printf(" %s %s %s %s\033[K\n",
			FormatStatusPill(change.PrevStatus),
			arrow,
			FormatStatusPill(change.Status),
			shortPath)
	}

	for i := len(changes) - startIdx; i < availableLines; i++ {
		fmt.Printf("\033[K\n")
	}
}

// Segment for progress bar
type Segment struct {
	Width float64
	Color RGB
}

func (pt *ProgressTracker) renderFullWidthProgressBar(current uint32, width int) {
	barWidth := width - 25
	if barWidth < 20 {
		barWidth = 20
	}

	passed := pt.passed.Load()
	failed := pt.failed.Load()
	skipped := pt.skipped.Load()
	timeout := pt.timeout.Load()
	crashed := pt.crashed.Load()
	other := current - passed - failed - skipped - timeout - crashed
	remaining := pt.totalTests - current

	total := float64(pt.totalTests)
	if total == 0 {
		total = 1
	}

	segments := []Segment{
		{float64(passed) / total * float64(barWidth), colorPass},
		{float64(failed) / total * float64(barWidth), colorFail},
		{float64(timeout) / total * float64(barWidth), colorTimeout},
		{float64(crashed) / total * float64(barWidth), colorCrash},
		{float64(skipped) / total * float64(barWidth), colorSkip},
		{float64(other) / total * float64(barWidth), colorOther},
		{float64(remaining) / total * float64(barWidth), colorEmpty},
	}

	bar := renderSegmentedBar(segments)
	percentage := float64(current) / float64(pt.totalTests) * 100

	fmt.Printf("[%s] %5.1f%% (%d/%d)\033[K\n", bar, percentage, current, pt.totalTests)
}

func renderSegmentedBar(segments []Segment) string {
	var result strings.Builder

	for i, seg := range segments {
		if seg.Width <= 0 {
			continue
		}

		fullBlocks := int(seg.Width)
		fractional := seg.Width - float64(fullBlocks)

		for b := 0; b < fullBlocks; b++ {
			// mild gradient inside each segment
			var t float64
			if fullBlocks > 1 {
				t = float64(b) / float64(fullBlocks-1)
			}

			c := shadeRGB(seg.Color, 0.92+0.14*t)
			fg := fmt.Sprintf("\033[38;2;%d;%d;%dm", c.R, c.G, c.B)
			result.WriteString(fg)
			result.WriteString(BlockFull)
			result.WriteString(ColorReset)
		}

		if fractional >= 0.125 {
			nextColor := colorEmpty
			for j := i + 1; j < len(segments); j++ {
				if segments[j].Width > 0 {
					nextColor = segments[j].Color
					break
				}
			}

			// Blend the edge block between segment colors
			a := rgbToHSV(seg.Color)
			b := rgbToHSV(nextColor)
			edge := hsvToRGB(blendHSV(a, b, 0.5))

			fgColor := fmt.Sprintf("\033[38;2;%d;%d;%dm", edge.R, edge.G, edge.B)
			bgColor := fmt.Sprintf("\033[48;2;%d;%d;%dm", nextColor.R, nextColor.G, nextColor.B)

			result.WriteString(fgColor)
			result.WriteString(bgColor)
			result.WriteString(getFractionalBlock(fractional))
			result.WriteString(ColorReset)
		}
	}

	return result.String()
}

func getFractionalBlock(fraction float64) string {
	switch {
	case fraction >= 0.875:
		return Block7_8
	case fraction >= 0.75:
		return Block6_8
	case fraction >= 0.625:
		return Block5_8
	case fraction >= 0.5:
		return Block4_8
	case fraction >= 0.375:
		return Block3_8
	case fraction >= 0.25:
		return Block2_8
	case fraction >= 0.125:
		return Block1_8
	default:
		return ""
	}
}

func (pt *ProgressTracker) renderStatusLine(label string, count uint32, total uint32, gained, lost int32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)
	delta := formatDelta(gained, lost)

	fmt.Printf(" %s %6d (%5.2f%%) %s\033[K\n", pill, count, percentage, delta)
}

func (pt *ProgressTracker) renderStatusLineSimple(label string, count uint32, total uint32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)

	fmt.Printf(" %s %6d (%5.2f%%)\033[K\n", pill, count, percentage)
}

func (pt *ProgressTracker) printProgressBar(current uint32) {
	passed := pt.passed.Load()
	failed := pt.failed.Load()
	skipped := pt.skipped.Load()
	timeout := pt.timeout.Load()
	crashed := pt.crashed.Load()
	other := current - passed - failed - skipped - timeout - crashed

	barWidth := 50
	total := float64(pt.totalTests)
	if total == 0 {
		total = 1
	}

	segments := []Segment{
		{float64(passed) / total * float64(barWidth), colorPass},
		{float64(failed) / total * float64(barWidth), colorFail},
		{float64(timeout) / total * float64(barWidth), colorTimeout},
		{float64(crashed) / total * float64(barWidth), colorCrash},
		{float64(skipped) / total * float64(barWidth), colorSkip},
		{float64(other) / total * float64(barWidth), colorOther},
		{float64(pt.totalTests-current) / total * float64(barWidth), colorEmpty},
	}

	bar := renderSegmentedBar(segments)
	percentage := float64(current) / float64(pt.totalTests) * 100

	fmt.Printf("\r[%s] %3d%% (%d/%d)", bar, int(percentage), current, pt.totalTests)
}

func PrintSummary(s Summary) {
	fmt.Printf("\n%s=== Final Results ===%s\n\n", ColorBold, ColorReset)

	printFinalStatusLine("PASS", s.Passed, s.Total, s.PassGained, s.PassLost, pillStylePass)
	printFinalStatusLine("FAIL", s.Failed, s.Total, s.FailGained, s.FailLost, pillStyleFail)
	printFinalStatusLine("SKIP", s.Skipped, s.Total, s.SkipGained, s.SkipLost, pillStyleSkip)
	printFinalStatusLine("TIMEOUT", s.Timeout, s.Total, s.TimeoutGained, s.TimeoutLost, pillStyleTimeout)
	printFinalStatusLine("CRASH", s.Crashed, s.Total, s.CrashGained, s.CrashLost, pillStyleCrash)
	printFinalStatusLine("PARSE_ERR", s.ParseError, s.Total, s.ParseErrGained, s.ParseErrLost, pillStyleParseError)
	printFinalStatusLine("PARSE_SUC", s.ParseSuccessError, s.Total, s.ParseSucGained, s.ParseSucLost, pillStyleParseError)
	printFinalStatusLine("NOT_IMPL", s.NotImplemented, s.Total, s.NotImplGained, s.NotImplLost, pillStyleNotImpl)
	printFinalStatusLine("RUN_ERR", s.RunnerError, s.Total, s.RunErrGained, s.RunErrLost, pillStyleRunnerError)

	fmt.Printf("\n%sTotal: %d%s\n", ColorBold, s.Total, ColorReset)
}

func printFinalStatusLine(label string, count uint32, total uint32, gained, lost int32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)
	delta := formatDelta(gained, lost)

	fmt.Printf("%s %6d (%5.2f%%) %s\n", pill, count, percentage, delta)
}

func printFinalStatusLineSimple(label string, count uint32, total uint32, style PillStyle) {
	printFinalStatusLine(label, count, total, 0, 0, style)
}

func (pt *ProgressTracker) GetStats() (passed, failed, skipped, timeout, crashed, parseError, parseSuccessError, notImplemented, runnerError, total uint32) {
	return pt.passed.Load(), pt.failed.Load(), pt.skipped.Load(), pt.timeout.Load(), pt.crashed.Load(),
		pt.parseError.Load(), pt.parseSuccessError.Load(), pt.notImplemented.Load(), pt.runnerError.Load(),
		pt.total.Load()
}
