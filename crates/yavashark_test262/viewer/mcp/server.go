package mcp

import (
	"fmt"
	"log"
	"net/http"
	"viewer/conf"
	"viewer/scripting"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

var server *mcp.Server
var runtime *scripting.Runtime

// Initialize creates and configures the MCP server
func Initialize() error {
	server = mcp.NewServer(
		&mcp.Implementation{
			Name:    "yavashark-test262",
			Version: "v2.0.0",
			Title:   "YavaShark Test262 Viewer",
		},
		nil,
	)

	// Add tools
	addTools(server)

	// Initialize Deno runtime
	runtime = scripting.NewRuntime()
	if err := runtime.Start(); err != nil {
		return fmt.Errorf("failed to start Deno runtime: %w", err)
	}

	log.Printf("MCP server initialized")
	return nil
}

// GetHandler returns an HTTP handler for MCP requests
func GetHandler() http.Handler {
	if server == nil {
		log.Fatal("MCP server not initialized")
	}

	return mcp.NewStreamableHTTPHandler(func(req *http.Request) *mcp.Server {
		return server
	}, nil)
}

// GetServer returns the MCP server instance
func GetServer() *mcp.Server {
	return server
}

// GetRuntime returns the Deno runtime instance
func GetRuntime() *scripting.Runtime {
	return runtime
}

// GetServerURL returns the URL for the API server
func GetServerURL() string {
	return fmt.Sprintf("http://localhost:%d", conf.Port)
}

// Shutdown stops the MCP server and Deno runtime
func Shutdown() {
	if runtime != nil {
		runtime.Stop()
	}
}
