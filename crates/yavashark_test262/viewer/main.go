package main

import (
	"log"
	"viewer/cache"
	"viewer/conf"
	"viewer/mcp"
	"viewer/router"

	"github.com/gofiber/fiber/v2/middleware/adaptor"
)

func main() {
	// Parse command line flags
	conf.ParseFlags()

	// Check MCP requirements if MCP is enabled
	if errMsg := conf.CheckMCPRequirements(); errMsg != "" {
		log.Fatalf("MCP requirement check failed: %s", errMsg)
	}

	// Initialize cache
	if err := cache.InitWithCurrent(); err != nil {
		log.Fatalf("Failed to initialize cache: %v", err)
	}

	// Set up HTTP router (without static files yet)
	app := router.Setup()

	// Initialize and add MCP routes if enabled (BEFORE static file serving)
	if conf.MCPEnabled {
		if err := mcp.Initialize(); err != nil {
			log.Fatalf("Failed to initialize MCP server: %v", err)
		}

		// Mount MCP handler at /mcp path
		mcpHandler := mcp.GetHandler()
		app.All("/mcp", adaptor.HTTPHandler(mcpHandler))
		app.All("/mcp/*", adaptor.HTTPHandler(mcpHandler))

		log.Printf("MCP server enabled at /mcp (Deno: %s)", conf.DenoPath)
	}

	// Now serve static files (includes catch-all route)
	router.ServeStatic()

	// Start the server
	router.Start()
}
