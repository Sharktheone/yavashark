package router

import (
	"viewer/runhistory"

	"github.com/gofiber/fiber/v2"
)

// Re-export types for backward compatibility with stream.go
type ChangedTest = runhistory.ChangedTest
type RunHistoryEntry = runhistory.RunHistoryEntry

// Re-export function for backward compatibility
var AddRunToHistory = runhistory.AddRunToHistory

func getRunHistory(c *fiber.Ctx) error {
	history, err := runhistory.LoadHistory()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to load history: " + err.Error(),
		})
	}

	return c.JSON(history)
}

func getRunHistoryEntry(c *fiber.Ctx) error {
	id := c.Params("id")
	if id == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Run ID is required",
		})
	}

	entry, err := runhistory.GetRunEntry(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "Run not found",
		})
	}

	return c.JSON(entry)
}

func getRunHistoryDetails(c *fiber.Ctx) error {
	id := c.Params("id")
	if id == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Run ID is required",
		})
	}

	details, err := runhistory.LoadRunDetails(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "Run details not found: " + err.Error(),
		})
	}

	return c.JSON(details)
}

func deleteRunHistoryEntry(c *fiber.Ctx) error {
	id := c.Params("id")
	if id == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Run ID is required",
		})
	}

	if err := runhistory.DeleteRun(id); err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "Run not found",
		})
	}

	return c.JSON(fiber.Map{
		"status": "deleted",
		"id":     id,
	})
}

func clearRunHistory(c *fiber.Ctx) error {
	if err := runhistory.ClearHistory(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to clear history: " + err.Error(),
		})
	}

	return c.JSON(fiber.Map{
		"status": "cleared",
	})
}
