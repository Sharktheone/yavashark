package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"viewer/conf"

	"github.com/google/uuid"
	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// Parameter types for tools

type GetApiDocsParams struct{}

type GetSessionParams struct{}

type RunScriptParams struct {
	Script    string `json:"script" jsonschema:"TypeScript code to execute. Has access to the ys namespace with tests, harness, spec, runner, and session APIs."`
	SessionId string `json:"sessionId,omitempty" jsonschema:"Optional session ID for state persistence between calls. Get one from GetSession."`
}

type SetMaxOutputCharsParams struct {
	MaxChars int `json:"maxChars" jsonschema:"Maximum number of characters for tool output. Set to 0 for unlimited. Default is 50000."`
}

type GetMaxOutputCharsParams struct{}

// addTools registers all MCP tools with the server
func addTools(s *mcp.Server) {
	// GetApiDocs - Returns TypeScript API documentation
	mcp.AddTool(s, &mcp.Tool{
		Name:        "GetApiDocs",
		Description: "Get TypeScript API docs and examples. Takes NO arguments - just call it.",
	}, handleGetApiDocs)

	// GetSession - Get or create a session for state persistence
	mcp.AddTool(s, &mcp.Tool{
		Name:        "GetSession",
		Description: "Get a new session ID for state persistence. Takes NO arguments - just call it.",
	}, handleGetSession)

	// RunScript - Execute TypeScript code
	mcp.AddTool(s, &mcp.Tool{
		Name:        "RunScript",
		Description: "Execute TypeScript code. Use ys.print() for output or return an object. Only pass 'script' argument (required) and optionally 'sessionId'.",
	}, handleRunScript)

	// SetMaxOutputChars - Set the maximum output characters for tool responses
	mcp.AddTool(s, &mcp.Tool{
		Name:        "SetMaxOutputChars",
		Description: "Set the maximum number of characters returned in tool output. Useful to prevent excessive output from consuming context. Default is 50000. Set to 0 for unlimited. You can also use ys.output.configure() in scripts for more control (offset, head/tail mode).",
	}, handleSetMaxOutputChars)

	// GetMaxOutputChars - Get the current maximum output characters setting
	mcp.AddTool(s, &mcp.Tool{
		Name:        "GetMaxOutputChars",
		Description: "Get the current maximum output characters setting. Takes NO arguments - just call it. For full config including offset and mode, use ys.output.getConfig() in a script.",
	}, handleGetMaxOutputChars)
}

func handleGetApiDocs(ctx context.Context, req *mcp.CallToolRequest, args GetApiDocsParams) (*mcp.CallToolResult, any, error) {
	docs := getTypeScriptAPIDocs()
	examples := getExampleScripts()

	text := "# YavaShark Test262 TypeScript API\n\n" + docs + "\n\n# Example Scripts\n\n" + examples

	return &mcp.CallToolResult{
		Content: []mcp.Content{&mcp.TextContent{Text: text}},
	}, nil, nil
}

func handleGetSession(ctx context.Context, req *mcp.CallToolRequest, args GetSessionParams) (*mcp.CallToolResult, any, error) {
	sessionID := uuid.New().String()
	return respondWith(map[string]string{"sessionId": sessionID}), nil, nil
}

func handleSetMaxOutputChars(ctx context.Context, req *mcp.CallToolRequest, args SetMaxOutputCharsParams) (*mcp.CallToolResult, any, error) {
	if args.MaxChars < 0 {
		return &mcp.CallToolResult{
			Content: []mcp.Content{&mcp.TextContent{Text: "maxChars must be >= 0 (0 means unlimited)"}},
			IsError: true,
		}, nil, nil
	}
	conf.MaxOutputChars = args.MaxChars
	msg := fmt.Sprintf("Max output chars set to %d", args.MaxChars)
	if args.MaxChars == 0 {
		msg = "Max output chars set to unlimited"
	}
	return respondWith(map[string]any{"maxChars": args.MaxChars, "message": msg}), nil, nil
}

func handleGetMaxOutputChars(ctx context.Context, req *mcp.CallToolRequest, args GetMaxOutputCharsParams) (*mcp.CallToolResult, any, error) {
	return respondWith(map[string]any{
		"maxChars":  conf.MaxOutputChars,
		"unlimited": conf.MaxOutputChars == 0,
	}), nil, nil
}

func handleRunScript(ctx context.Context, req *mcp.CallToolRequest, args RunScriptParams) (*mcp.CallToolResult, any, error) {
	runtime := GetRuntime()
	if runtime == nil || !runtime.IsRunning() {
		return &mcp.CallToolResult{
			Content: []mcp.Content{&mcp.TextContent{Text: "Deno runtime is not available"}},
			IsError: true,
		}, nil, nil
	}

	serverURL := GetServerURL()
	result, err := runtime.Execute(ctx, args.Script, args.SessionId, serverURL)
	if err != nil {
		return &mcp.CallToolResult{
			Content: []mcp.Content{&mcp.TextContent{Text: "Script execution failed: " + err.Error()}},
			IsError: true,
		}, nil, nil
	}

	if !result.Success {
		return &mcp.CallToolResult{
			Content: []mcp.Content{&mcp.TextContent{Text: "Script error: " + result.Error}},
			IsError: true,
		}, nil, nil
	}

	// result.Result is already json.RawMessage from Deno
	// Use it directly for text content, but structured content must be an object
	resultText := string(result.Result)
	if resultText == "" || resultText == "null" || resultText == "undefined" {
		resultText = "{}"
		result.Result = json.RawMessage(resultText)
	}

	// StructuredContent must be a JSON object (not array or primitive)
	// Check if result is an object, if not wrap it
	var structuredContent json.RawMessage
	trimmed := []byte(resultText)
	if len(trimmed) > 0 && trimmed[0] == '{' {
		// Already an object, use as-is
		structuredContent = result.Result
	} else {
		// Wrap in object with "value" key
		wrapped := map[string]json.RawMessage{"value": result.Result}
		structuredContent, _ = json.Marshal(wrapped)
	}

	// Apply truncation to text output
	displayText := truncateOutput(resultText)

	return &mcp.CallToolResult{
		Content:           []mcp.Content{&mcp.TextContent{Text: displayText}},
		StructuredContent: structuredContent,
	}, nil, nil
}

// respondWith creates a tool result with JSON content
func respondWith(res any) *mcp.CallToolResult {
	b, _ := json.Marshal(res)
	text := truncateOutput(string(b))
	return &mcp.CallToolResult{
		StructuredContent: json.RawMessage(b),
		Content:           []mcp.Content{&mcp.TextContent{Text: text}},
	}
}

// truncateOutput truncates the output text based on configured settings
func truncateOutput(text string) string {
	maxChars := conf.MaxOutputChars
	offset := conf.OutputOffset
	mode := conf.OutputMode

	totalLen := len(text)

	// Record the length before truncation
	conf.LastOutputLength = totalLen

	// No truncation if unlimited
	if maxChars <= 0 && offset <= 0 {
		return text
	}

	// Apply offset and maxChars based on mode
	var start, end int
	if mode == "tail" {
		// Take from end: start from (totalLen - offset - maxChars) to (totalLen - offset)
		end = totalLen - offset
		if end < 0 {
			end = 0
		}
		if maxChars > 0 && end > maxChars {
			start = end - maxChars
		} else {
			start = 0
		}
	} else {
		// "head" mode (default): start from offset, take maxChars
		start = offset
		if start > totalLen {
			start = totalLen
		}
		if maxChars > 0 {
			end = start + maxChars
			if end > totalLen {
				end = totalLen
			}
		} else {
			end = totalLen
		}
	}

	// No truncation needed
	if start == 0 && end == totalLen {
		return text
	}

	truncated := text[start:end]

	// Build truncation message
	var msg string
	if start > 0 && end < totalLen {
		msg = fmt.Sprintf("\n\n[Output truncated: showing chars %d-%d of %d total. Use ys.output.configure() to adjust.]", start, end, totalLen)
	} else if start > 0 {
		msg = fmt.Sprintf("\n\n[Output truncated: skipped first %d chars, showing %d of %d total. Use ys.output.configure() to adjust.]", start, end-start, totalLen)
	} else {
		msg = fmt.Sprintf("\n\n[Output truncated: %d chars shown of %d total. Use ys.output.configure() to adjust.]", end, totalLen)
	}

	return truncated + msg
}

// getTypeScriptAPIDocs returns the TypeScript API documentation
func getTypeScriptAPIDocs() string {
	return "## Available APIs\n\n" +
		"The `ys` namespace provides access to all functionality:\n\n" +
		"### ys.print() - Output text\n\n" +
		"```typescript\n" +
		"ys.print(...args: unknown[]): void   // Print to output (like console.log)\n" +
		"```\n\n" +
		"Use `ys.print()` to output results. You can also `return` a value.\n" +
		"If you use both, the result will contain both `output` and `result` fields.\n\n" +
		"### ys.tests - Query and manipulate tests\n\n" +
		"```typescript\n" +
		"// Query builders\n" +
		"ys.tests.all(): TestQuery                           // All tests\n" +
		"ys.tests.inDir(path: string, recursive?: boolean)   // Tests in directory\n" +
		"ys.tests.search(query: string): TestQuery           // Search by path\n" +
		"ys.tests.withStatus(status: Status): TestQuery      // Filter by status\n" +
		"ys.tests.failing(): TestQuery                       // Shorthand for failed tests\n\n" +
		"// Direct access\n" +
		"ys.tests.getStatus(path: string): Promise<TestStatus>\n" +
		"ys.tests.getOutput(path: string): Promise<TestOutput>\n" +
		"ys.tests.getCode(path: string): Promise<string>\n" +
		"ys.tests.setCode(path: string, code: string): Promise<void>  // In-memory only\n" +
		"```\n\n" +
		"### TestQuery methods\n\n" +
		"```typescript\n" +
		"interface TestQuery {\n" +
		"  filter(predicate: (t: TestEntry) => boolean): TestQuery\n" +
		"  withStatus(status: Status | Status[]): TestQuery\n" +
		"  inDir(path: string): TestQuery\n\n" +
		"  // Terminal operations\n" +
		"  toArray(): Promise<TestEntry[]>\n" +
		"  count(): Promise<number>\n" +
		"  first(n?: number): Promise<TestEntry[]>\n" +
		"  paths(): Promise<string[]>\n\n" +
		"  // Aggregations\n" +
		"  groupByStatus(): Promise<Record<Status, TestEntry[]>>\n" +
		"  groupByDir(depth?: number): Promise<Record<string, TestEntry[]>>\n" +
		"}\n" +
		"```\n\n" +
		"### ys.harness - Access test harness files\n\n" +
		"```typescript\n" +
		"ys.harness.getCode(name: string): Promise<string>\n" +
		"ys.harness.listForTest(testPath: string): Promise<string[]>\n" +
		"ys.harness.getForTest(testPath: string): Promise<Record<string, string>>\n" +
		"ys.harness.setCode(name: string, code: string): Promise<void>  // In-memory only\n" +
		"```\n\n" +
		"### ys.spec - Access ECMAScript specification\n\n" +
		"```typescript\n" +
		"ys.spec.get(section: string): Promise<string>\n" +
		"ys.spec.search(query: string): Promise<SpecMatch[]>\n" +
		"ys.spec.forIntrinsic(name: string): Promise<string>\n" +
		"```\n\n" +
		"### ys.runner - Run tests\n\n" +
		"```typescript\n" +
		"ys.runner.rerun(opts: RerunOptions): Promise<RerunResult>      // Blocking\n" +
		"ys.runner.rerunAsync(opts: RerunOptions): Promise<RerunJob>    // Non-blocking\n" +
		"ys.runner.getJob(jobId: string): Promise<RerunJob>\n" +
		"ys.runner.cancelJob(jobId: string): Promise<void>\n\n" +
		"interface RerunOptions {\n" +
		"  paths?: string[]       // Specific test paths\n" +
		"  dir?: string           // Directory to run\n" +
		"  failedOnly?: boolean   // Only run failed tests\n" +
		"  rebuild?: boolean      // Rebuild engine first\n" +
		"}\n" +
		"```\n\n" +
		"### ys.session - Persist state between calls\n\n" +
		"```typescript\n" +
		"ys.session.get<T>(key: string): T | undefined\n" +
		"ys.session.set<T>(key: string, value: T): void\n" +
		"ys.session.delete(key: string): void\n" +
		"ys.session.clear(): void\n" +
		"```\n\n" +
		"### ys.output - Control output truncation\n\n" +
		"```typescript\n" +
		"// Set max characters (0 for unlimited, default: 50000)\n" +
		"ys.output.setMaxChars(maxChars: number): Promise<void>\n" +
		"ys.output.getMaxChars(): Promise<{ maxChars: number; unlimited: boolean }>\n\n" +
		"// Set character offset (skip first/last N chars)\n" +
		"ys.output.setOffset(offset: number): Promise<void>\n" +
		"ys.output.getOffset(): Promise<number>\n\n" +
		"// Set mode: 'head' (from start) or 'tail' (from end)\n" +
		"ys.output.setMode(mode: 'head' | 'tail'): Promise<void>\n" +
		"ys.output.getMode(): Promise<'head' | 'tail'>\n\n" +
		"// Configure all at once\n" +
		"ys.output.configure(config: {\n" +
		"  maxChars?: number;   // Max chars to return (0 = unlimited)\n" +
		"  offset?: number;     // Chars to skip\n" +
		"  mode?: 'head' | 'tail';  // Take from start or end\n" +
		"}): Promise<void>\n\n" +
		"ys.output.getConfig(): Promise<{\n" +
		"  maxChars: number;\n" +
		"  offset: number;\n" +
		"  mode: 'head' | 'tail';\n" +
		"  unlimited: boolean;\n" +
		"}>\n\n" +
		"// Get length of last output (before truncation)\n" +
		"ys.output.getLastLength(): Promise<number>\n" +
		"```\n\n" +
		"### Types\n\n" +
		"```typescript\n" +
		"type Status = 'PASS' | 'FAIL' | 'SKIP' | 'TIMEOUT' | 'CRASH' | 'PARSE_ERROR' | 'NOT_IMPLEMENTED'\n\n" +
		"interface TestEntry {\n" +
		"  path: string\n" +
		"  status: Status\n" +
		"}\n\n" +
		"interface TestStatus {\n" +
		"  path: string\n" +
		"  status: Status\n" +
		"}\n\n" +
		"interface TestOutput {\n" +
		"  path: string\n" +
		"  status: Status\n" +
		"  message: string\n" +
		"  duration: number\n" +
		"}\n\n" +
		"interface RerunResult {\n" +
		"  before: TestEntry[]\n" +
		"  after: TestEntry[]\n" +
		"  diff: DiffResult\n" +
		"  duration: number\n" +
		"  status: 'complete' | 'timeout' | 'cancelled'\n" +
		"}\n\n" +
		"interface DiffResult {\n" +
		"  gained: TestEntry[]   // Now passing\n" +
		"  lost: TestEntry[]     // Now failing\n" +
		"  changed: TestEntry[]  // Status changed (any)\n" +
		"}\n" +
		"```\n"
}

// getExampleScripts returns example scripts
func getExampleScripts() string {
	return "### Example 1: Simple count with print\n\n" +
		"```typescript\n" +
		"const count = await ys.tests.failing().count();\n" +
		"ys.print(`There are ${count} failing tests`);\n" +
		"```\n\n" +
		"### Example 2: Get failing tests in a directory\n\n" +
		"```typescript\n" +
		"const failing = await ys.tests\n" +
		"  .inDir(\"built-ins/Array/prototype/map\")\n" +
		"  .withStatus(\"FAIL\")\n" +
		"  .toArray();\n\n" +
		"return {\n" +
		"  count: failing.length,\n" +
		"  paths: failing.map(t => t.path)\n" +
		"};\n" +
		"```\n\n" +
		"### Example 2: Get output for first 5 failing tests\n\n" +
		"```typescript\n" +
		"const failing = await ys.tests.failing().first(5);\n" +
		"const outputs = await Promise.all(\n" +
		"  failing.map(t => ys.tests.getOutput(t.path))\n" +
		");\n\n" +
		"return outputs.map(o => ({\n" +
		"  path: o.path,\n" +
		"  message: o.message\n" +
		"}));\n" +
		"```\n\n" +
		"### Example 3: Rerun failing tests and compare\n\n" +
		"```typescript\n" +
		"const result = await ys.runner.rerun({\n" +
		"  dir: \"built-ins/Array\",\n" +
		"  failedOnly: true,\n" +
		"  rebuild: false\n" +
		"});\n\n" +
		"return {\n" +
		"  improved: result.diff.gained.length,\n" +
		"  regressed: result.diff.lost.length,\n" +
		"  duration: result.duration\n" +
		"};\n" +
		"```\n\n" +
		"### Example 4: Group tests by status\n\n" +
		"```typescript\n" +
		"const byStatus = await ys.tests\n" +
		"  .inDir(\"language/expressions\")\n" +
		"  .groupByStatus();\n\n" +
		"return Object.fromEntries(\n" +
		"  Object.entries(byStatus).map(([status, tests]) =>\n" +
		"    [status, tests.length]\n" +
		"  )\n" +
		");\n" +
		"```\n\n" +
		"### Example 5: Use session to track progress\n\n" +
		"```typescript\n" +
		"// Get or initialize our working set\n" +
		"let workingSet = ys.session.get<string[]>(\"failingTests\");\n" +
		"if (!workingSet) {\n" +
		"  const failing = await ys.tests.failing().paths();\n" +
		"  workingSet = failing.slice(0, 10); // Work on 10 at a time\n" +
		"  ys.session.set(\"failingTests\", workingSet);\n" +
		"}\n\n" +
		"// Get the first test to work on\n" +
		"const testPath = workingSet[0];\n" +
		"const output = await ys.tests.getOutput(testPath);\n" +
		"const code = await ys.tests.getCode(testPath);\n\n" +
		"return {\n" +
		"  currentTest: testPath,\n" +
		"  remaining: workingSet.length,\n" +
		"  output: output.message,\n" +
		"  codePreview: code.substring(0, 500)\n" +
		"};\n" +
		"```\n\n" +
		"### Example 6: Control output truncation\n\n" +
		"```typescript\n" +
		"// Limit output to 10000 chars\n" +
		"await ys.output.setMaxChars(10000);\n\n" +
		"// Get last 5000 chars of output (tail mode)\n" +
		"await ys.output.configure({ maxChars: 5000, mode: 'tail' });\n\n" +
		"// Skip first 1000 chars, get next 5000\n" +
		"await ys.output.configure({ maxChars: 5000, offset: 1000, mode: 'head' });\n\n" +
		"// Check current settings\n" +
		"const config = await ys.output.getConfig();\n" +
		"ys.print(`Max: ${config.maxChars}, Mode: ${config.mode}`);\n\n" +
		"// Paginate through large output\n" +
		"const len = await ys.output.getLastLength();\n" +
		"if (len > 10000) {\n" +
		"  ys.print(`Output was ${len} chars, fetching next page...`);\n" +
		"  await ys.output.setOffset(10000);  // Skip first 10k\n" +
		"}\n" +
		"```\n"
}
