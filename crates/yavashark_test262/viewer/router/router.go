package router

import (
	"fmt"
	"viewer/conf"
	"viewer/web"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
)

var app *fiber.App

// Setup creates and configures the Fiber app with all routes
func Setup() *fiber.App {
	app = fiber.New()

	app.Use(cors.New())
	app.Use(logger.New())
	app.Use(recover.New())

	api := app.Group("api")

	api.Get("capabilities", capabilities)
	api.Get("profiles", profiles)

	api.Get("current", current)
	api.Get("info/*", info)

	api.Get("rerun", rerunAll)
	api.Get("rerun/*", rerun)

	api.Get("rerun-stream", rerunStream)
	api.Get("rerun-stream/*", rerunStreamPath)

	api.Post("cancel", cancelRun)

	api.Get("history", getRunHistory)
	api.Get("history/:id", getRunHistoryEntry)
	api.Delete("history/:id", deleteRunHistoryEntry)
	api.Delete("history", clearRunHistory)

	api.Get("git/info", getCurrentGitInfo)
	api.Get("git/branches", getDataRepoBranches)
	api.Get("git/commits", getDataRepoCommits)
	api.Get("git/results/:hash", getResultsForCommit)

	// Script API call endpoint (used by Deno runtime)
	api.Post("script/call", scriptAPICall)

	return app
}

// ServeStatic sets up static file serving. Call this AFTER registering
// other routes (like MCP) to ensure proper route priority.
func ServeStatic() {
	web.Serve(app)
}

// GetApp returns the Fiber app instance
func GetApp() *fiber.App {
	return app
}

// Start begins listening on the configured port
func Start() {
	if app == nil {
		app = Setup()
	}

	addr := fmt.Sprintf(":%d", conf.Port)
	if err := app.Listen(addr); err != nil {
		panic(err)
	}
}
