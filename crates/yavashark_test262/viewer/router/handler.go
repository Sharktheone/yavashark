package router

import (
	"github.com/gofiber/fiber/v2"
	"path/filepath"
	"sync"
	"viewer/conf"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"
)

var (
	testRun = sync.Mutex{}
)

func current(c *fiber.Ctx) error {
	res, err := results.LoadResults()
	if err != nil {
		return err
	}

	if res == nil {
		return fiber.NewError(fiber.StatusNotFound, "No results found")
	}

	resCi := results.ConvertResultsToCI(res, conf.TestRoot)

	return c.Status(fiber.StatusOK).JSON(resCi)
}

func rerunAll(c *fiber.Ctx) error {
	if err := rebuild(); err != nil {
		return err
	}

	if !testRun.TryLock() {
		return fiber.NewError(fiber.StatusTooManyRequests, "Test is already running")
	}

	defer testRun.Unlock()

	res := run.TestsInDir(conf.TestRoot, conf.Workers)
	res.Write()

	resCi := results.ConvertResultsToCI(res.TestResults, conf.TestRoot)

	return c.Status(fiber.StatusOK).JSON(resCi)
}

func rerun(c *fiber.Ctx) error {
	if err := rebuild(); err != nil {
		return err
	}

	if !testRun.TryLock() {
		return fiber.NewError(fiber.StatusTooManyRequests, "Test is already running")
	}

	defer testRun.Unlock()

	path := c.Params("path")

	fullPath := filepath.Join(conf.TestRoot, path)

	run.TestsInDir(fullPath, conf.Workers)

	return current(c)
}

func info(c *fiber.Ctx) error {
	res, err := results.LoadResults()
	if err != nil {
		return err
	}

	path, err := filepath.Rel("/api/info/", c.Path())
	if err != nil {
		return err
	}
	fullPath := filepath.Join(conf.TestRoot, path)

	for _, r := range res {
		if r.Path == fullPath {
			return c.Status(fiber.StatusOK).JSON(r)
		}
	}

	return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
		"error": "Test not found",
	})
}
