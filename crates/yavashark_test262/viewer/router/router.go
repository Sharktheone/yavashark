package router

import (
	"viewer/web"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
)

func Start() {
	app := fiber.New()

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

	web.Serve(app)

	if err := app.Listen(":1215"); err != nil {
		panic(err)
	}
}
