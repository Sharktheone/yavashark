package router

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
	"viewer/web"
)

func Start() {
	app := fiber.New()

	app.Use(cors.New())
	app.Use(logger.New())
	app.Use(recover.New())

	api := app.Group("api")

	api.Get("current", current)
	api.Get("rerun", rerunAll)
	api.Get("rerun/*", rerun)
	api.Get("info/*", info)

	web.Serve(app)

	if err := app.Listen(":1215"); err != nil {
		panic(err)
	}
}
