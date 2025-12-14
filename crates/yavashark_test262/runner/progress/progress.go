package progress

import (
	"fmt"
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
		Right: RGB{129, 199, 132},
		Text:  RGB{27, 94, 32},
	}
	pillStyleFail = PillStyle{
		Left:  RGB{229, 57, 53},
		Right: RGB{239, 154, 154},
		Text:  RGB{255, 255, 255},
	}
	pillStyleSkip = PillStyle{
		Left:  RGB{0, 172, 193},
		Right: RGB{128, 222, 234},
		Text:  RGB{0, 77, 64},
	}
	pillStyleTimeout = PillStyle{
		Left:  RGB{255, 179, 0},
		Right: RGB{255, 224, 130},
		Text:  RGB{62, 39, 35},
	}
	pillStyleCrash = PillStyle{
		Left:  RGB{142, 36, 170},
		Right: RGB{206, 147, 216},
		Text:  RGB{255, 255, 255},
	}
	pillStyleParseError = PillStyle{
		Left:  RGB{30, 136, 229},
		Right: RGB{144, 202, 249},
		Text:  RGB{13, 71, 161},
	}
	pillStyleNotImpl = PillStyle{
		Left:  RGB{117, 117, 117},
		Right: RGB{189, 189, 189},
		Text:  RGB{33, 33, 33},
	}
	pillStyleRunnerError = PillStyle{
		Left:  RGB{97, 97, 97},
		Right: RGB{158, 158, 158},
		Text:  RGB{250, 250, 250},
	}
)

type RecentChange struct {
	Path       string
	Status     status.Status
	PrevStatus status.Status
	IsNew      bool
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

	totalTests  uint32
	interactive bool

	prevResults map[string]status.Status

	passGained atomic.Int32
	passLost   atomic.Int32
	failGained atomic.Int32
	failLost   atomic.Int32

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

	if interactive {
		pt.enterInteractiveMode()
	}

	return pt
}

func (pt *ProgressTracker) enterInteractiveMode() {
	fmt.Print("\033[?1049h")
	fmt.Print("\033[?25l")
	fmt.Print("\033[2J")
}

func (pt *ProgressTracker) exitInteractiveMode() {
	fmt.Print("\033[?25h")
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

			if pt.interactive && (s == status.PASS || s == status.FAIL || prevStatus == status.PASS || prevStatus == status.FAIL) {
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
			percentThreshold := pt.totalTests / 200
			if percentThreshold > threshold {
				threshold = percentThreshold
			}
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

func (pt *ProgressTracker) Finish() {
	if !pt.interactive {
		pt.printProgressBar(pt.total.Load())
		fmt.Println()
	}

	if pt.interactive {
		pt.renderInteractive(pt.total.Load())
		pt.exitInteractiveMode()
	}

	pt.printFinalSummary()
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
	paddedLabel := fmt.Sprintf("  %s  ", label)
	halfLen := len(paddedLabel) / 2
	leftPart := paddedLabel[:halfLen]
	rightPart := paddedLabel[halfLen:]

	leftBg := fmt.Sprintf("\033[48;2;%d;%d;%dm", style.Left.R, style.Left.G, style.Left.B)
	rightBg := fmt.Sprintf("\033[48;2;%d;%d;%dm", style.Right.R, style.Right.G, style.Right.B)
	textColor := fmt.Sprintf("\033[38;2;%d;%d;%dm", style.Text.R, style.Text.G, style.Text.B)

	return fmt.Sprintf("%s%s%s%s%s%s%s", leftBg, textColor, leftPart, rightBg, rightPart, ColorReset, "")
}

func formatStatusPill(s status.Status) string {
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

	fmt.Print("\033[H")

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

	pt.renderStatusLine("PASS", passed, pt.totalTests, pt.passGained.Load(), pt.passLost.Load(), pillStylePass)
	pt.renderStatusLine("FAIL", failed, pt.totalTests, pt.failGained.Load(), pt.failLost.Load(), pillStyleFail)
	pt.renderStatusLineSimple("SKIP", skipped, pt.totalTests, pillStyleSkip)
	pt.renderStatusLineSimple("TIMEOUT", timeout, pt.totalTests, pillStyleTimeout)
	pt.renderStatusLineSimple("CRASH", crashed, pt.totalTests, pillStyleCrash)
	pt.renderStatusLineSimple("PARSE_ERR", parseError, pt.totalTests, pillStyleParseError)
	pt.renderStatusLineSimple("PARSE_SUC", parseSuccessError, pt.totalTests, pillStyleParseError)
	pt.renderStatusLineSimple("NOT_IMPL", notImplemented, pt.totalTests, pillStyleNotImpl)
	pt.renderStatusLineSimple("RUN_ERR", runnerError, pt.totalTests, pillStyleRunnerError)

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

		fmt.Printf("  %s %s %s %s\033[K\n",
			formatStatusPill(change.PrevStatus),
			arrow,
			formatStatusPill(change.Status),
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

		fg := fmt.Sprintf("\033[38;2;%d;%d;%dm", seg.Color.R, seg.Color.G, seg.Color.B)

		if fullBlocks > 0 {
			result.WriteString(fg)
			result.WriteString(strings.Repeat(BlockFull, fullBlocks))
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

			fgColor := fmt.Sprintf("\033[38;2;%d;%d;%dm", seg.Color.R, seg.Color.G, seg.Color.B)
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

	fmt.Printf("%s %6d (%5.2f%%) %s\033[K\n", pill, count, percentage, delta)
}

func (pt *ProgressTracker) renderStatusLineSimple(label string, count uint32, total uint32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)

	fmt.Printf("%s %6d (%5.2f%%)\033[K\n", pill, count, percentage)
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

func (pt *ProgressTracker) printFinalSummary() {
	passed := pt.passed.Load()
	failed := pt.failed.Load()
	skipped := pt.skipped.Load()
	timeout := pt.timeout.Load()
	crashed := pt.crashed.Load()
	parseError := pt.parseError.Load()
	parseSuccessError := pt.parseSuccessError.Load()
	notImplemented := pt.notImplemented.Load()
	runnerError := pt.runnerError.Load()
	total := pt.total.Load()

	fmt.Printf("\n%s=== Final Results ===%s\n\n", ColorBold, ColorReset)

	pt.printFinalStatusLine("PASS", passed, total, pt.passGained.Load(), pt.passLost.Load(), pillStylePass)
	pt.printFinalStatusLine("FAIL", failed, total, pt.failGained.Load(), pt.failLost.Load(), pillStyleFail)
	pt.printFinalStatusLineSimple("SKIP", skipped, total, pillStyleSkip)
	pt.printFinalStatusLineSimple("TIMEOUT", timeout, total, pillStyleTimeout)
	pt.printFinalStatusLineSimple("CRASH", crashed, total, pillStyleCrash)
	pt.printFinalStatusLineSimple("PARSE_ERR", parseError, total, pillStyleParseError)
	pt.printFinalStatusLineSimple("PARSE_SUC", parseSuccessError, total, pillStyleParseError)
	pt.printFinalStatusLineSimple("NOT_IMPL", notImplemented, total, pillStyleNotImpl)
	pt.printFinalStatusLineSimple("RUN_ERR", runnerError, total, pillStyleRunnerError)

	fmt.Printf("\n%sTotal: %d%s\n", ColorBold, total, ColorReset)
}

func (pt *ProgressTracker) printFinalStatusLine(label string, count uint32, total uint32, gained, lost int32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)
	delta := formatDelta(gained, lost)

	fmt.Printf("%s %6d (%5.2f%%) %s\n", pill, count, percentage, delta)
}

func (pt *ProgressTracker) printFinalStatusLineSimple(label string, count uint32, total uint32, style PillStyle) {
	percentage := float64(count) / float64(total) * 100
	pill := formatPill(label, style)

	fmt.Printf("%s %6d (%5.2f%%)\n", pill, count, percentage)
}

func (pt *ProgressTracker) GetStats() (passed, failed, skipped, timeout, crashed, parseError, parseSuccessError, notImplemented, runnerError, total uint32) {
	return pt.passed.Load(), pt.failed.Load(), pt.skipped.Load(), pt.timeout.Load(), pt.crashed.Load(),
		pt.parseError.Load(), pt.parseSuccessError.Load(), pt.notImplemented.Load(), pt.runnerError.Load(),
		pt.total.Load()
}
