package web

import (
	"embed"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/filesystem"
	"io/fs"
	"log"
	"net/http"
	"strings"
)

var (
	//go:embed dist
	embedFS embed.FS
)

func Serve(a *fiber.App) {
	subFs, err := fs.Sub(embedFS, "dist")
	if err != nil {
		log.Fatalf("Failed to get subFS: %v", err)
	}
	FS := http.FS(subFs)

	static := filesystem.New(filesystem.Config{
		Root:   FS,
		Index:  "index.html",
		Browse: true,
		MaxAge: 3600,
	})

	a.Use(static)

	a.Get("*", func(c *fiber.Ctx) error {
		if strings.HasPrefix(c.Path(), "/api") {
			return c.Next()
		}
		return filesystem.SendFile(c, FS, "index.html")
	})

}
