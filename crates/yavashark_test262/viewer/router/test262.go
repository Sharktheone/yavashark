package router

import (
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"viewer/conf"

	"github.com/gofiber/fiber/v2"
)

func serveTest262File(c *fiber.Ctx) error {
	test262Root, err := conf.ResolveTest262Root()
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	relPath, err := url.PathUnescape(c.Params("*"))
	if err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid test262 path",
		})
	}

	relPath = strings.TrimPrefix(filepath.Clean("/"+relPath), "/")
	if relPath == "." || relPath == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "test262 path is required",
		})
	}

	fullPath := filepath.Join(test262Root, relPath)
	relToRoot, err := filepath.Rel(test262Root, fullPath)
	if err != nil || relToRoot == ".." || strings.HasPrefix(relToRoot, ".."+string(os.PathSeparator)) {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid test262 path",
		})
	}

	if info, err := os.Stat(fullPath); err != nil {
		if os.IsNotExist(err) {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "test262 file not found",
			})
		}

		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to read test262 file",
		})
	} else if info.IsDir() {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "test262 path must point to a file",
		})
	}

	return c.SendFile(fullPath)
}
