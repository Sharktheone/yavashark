package conf

import (
	"flag"
	"os"
	"os/exec"
)

// Default values
const (
	DefaultTestRoot         = "../../test262/test"
	DefaultWorkers          = 256
	DefaultRunnerPath       = "../runner"
	DefaultPort             = 1215
	DefaultMCPEnabled       = true
	DefaultScriptTimeout    = 10 // seconds
	DefaultMaxScriptTimeout = 30 // seconds
)

// Runtime configuration (set from flags or defaults)
var (
	TestRoot         string
	Workers          int
	RunnerPath       string
	Port             int
	MCPEnabled       bool
	ScriptTimeout    int
	MaxScriptTimeout int
	DenoPath         string
	DenoAvailable    bool
)

func init() {
	flag.StringVar(&TestRoot, "test-root", DefaultTestRoot, "Path to test262 test directory")
	flag.IntVar(&Workers, "workers", DefaultWorkers, "Number of parallel test workers")
	flag.StringVar(&RunnerPath, "runner-path", DefaultRunnerPath, "Path to runner directory")
	flag.IntVar(&Port, "port", DefaultPort, "HTTP server port")
	flag.BoolVar(&MCPEnabled, "mcp", DefaultMCPEnabled, "Enable MCP server")
	flag.IntVar(&ScriptTimeout, "script-timeout", DefaultScriptTimeout, "Default script execution timeout in seconds")
	flag.IntVar(&MaxScriptTimeout, "max-script-timeout", DefaultMaxScriptTimeout, "Maximum script execution timeout in seconds")
	flag.StringVar(&DenoPath, "deno-path", "", "Path to Deno executable (auto-detected if empty)")
}

// ParseFlags parses command line flags and initializes configuration
func ParseFlags() {
	flag.Parse()

	// Auto-detect Deno if not specified
	if DenoPath == "" {
		if path, err := exec.LookPath("deno"); err == nil {
			DenoPath = path
			DenoAvailable = true
		}
	} else {
		// Check if specified path exists
		if _, err := os.Stat(DenoPath); err == nil {
			DenoAvailable = true
		}
	}
}

// CheckMCPRequirements checks if MCP can be enabled
// Returns an error message if MCP cannot be enabled, empty string if OK
func CheckMCPRequirements() string {
	if !MCPEnabled {
		return ""
	}

	if !DenoAvailable {
		return "MCP server requires Deno to be installed. Install Deno or disable MCP with --mcp=false"
	}

	return ""
}
