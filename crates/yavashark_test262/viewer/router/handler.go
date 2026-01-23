package router

import (
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
	"viewer/cache"
	"viewer/conf"
	"yavashark_test262_runner/build"
	"yavashark_test262_runner/results"
	"yavashark_test262_runner/run"

	"github.com/gofiber/fiber/v2"
)

type Capabilities struct {
	CanRerun   bool     `json:"canRerun"`
	CanRebuild bool     `json:"canRebuild"`
	Profiles   []string `json:"profiles"`
}

func current(c *fiber.Ctx) error {
	resCi, err := cache.GetCi()

	if err != nil {
		return err
	}

	return c.Status(fiber.StatusOK).JSON(resCi)
}

func capabilities(c *fiber.Ctx) error {
	canRebuild := checkCargoAvailable()

	profileNames := getProfileNames()

	caps := Capabilities{
		CanRerun:   true,
		CanRebuild: canRebuild,
		Profiles:   profileNames,
	}

	return c.Status(fiber.StatusOK).JSON(caps)
}

func checkCargoAvailable() bool {
	_, err := exec.LookPath("cargo")
	return err == nil
}

func getProfileNames() []string {
	profilesPath := filepath.Join(conf.RunnerPath, "profiles.json")
	data, err := os.ReadFile(profilesPath)
	if err != nil {
		data, err = os.ReadFile("profiles.json")
		if err != nil {
			return []string{}
		}
	}

	var profilesConfig struct {
		Profiles map[string]any `json:"profiles"`
	}

	if err := json.Unmarshal(data, &profilesConfig); err != nil {
		return []string{}
	}

	names := make([]string, 0, len(profilesConfig.Profiles))
	for name := range profilesConfig.Profiles {
		names = append(names, name)
	}

	return names
}

func profiles(c *fiber.Ctx) error {
	profilesPath := filepath.Join(conf.RunnerPath, "profiles.json")
	data, err := os.ReadFile(profilesPath)
	if err != nil {
		data, err = os.ReadFile("profiles.json")
		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "profiles.json not found",
			})
		}
	}

	var profilesConfig any
	if err := json.Unmarshal(data, &profilesConfig); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to parse profiles.json",
		})
	}

	return c.Status(fiber.StatusOK).JSON(profilesConfig)
}

func rerunAll(c *fiber.Ctx) error {
	rebuildFlag := c.Query("rebuild", "true") == "true"

	if rebuildFlag {
		if err := build.RebuildEngine(build.Config{
			Rebuild:  true,
			Mode:     build.BuildModeRelease,
			Compiler: build.CompilerLLVM,
		}); err != nil {
			return fiber.NewError(fiber.StatusInternalServerError, "Build failed: "+err.Error())
		}
	}

	if !conf.TestRunLock.TryLock() {
		return fiber.NewError(fiber.StatusTooManyRequests, "Test is already running")
	}

	defer conf.TestRunLock.Unlock()

	runConfig := run.RunConfig{
		Workers:     conf.Workers,
		Skips:       true,
		Timings:     false,
		Timeout:     30 * time.Second,
		Interactive: false,
	}

	res, _ := run.TestsInDir(conf.TestRoot, runConfig)
	res.Write()

	resCi := results.ConvertResultsToCI(res.TestResults, conf.TestRoot)

	return c.Status(fiber.StatusOK).JSON(resCi)
}

func rerun(c *fiber.Ctx) error {
	rebuildFlag := c.Query("rebuild", "true") == "true"

	if rebuildFlag {
		if err := build.RebuildEngine(build.Config{
			Rebuild:  true,
			Mode:     build.BuildModeRelease,
			Compiler: build.CompilerLLVM,
		}); err != nil {
			return fiber.NewError(fiber.StatusInternalServerError, "Build failed: "+err.Error())
		}
	}

	if !conf.TestRunLock.TryLock() {
		return fiber.NewError(fiber.StatusTooManyRequests, "Test is already running")
	}

	defer conf.TestRunLock.Unlock()

	path, err := filepath.Rel("/api/rerun/", c.Path())
	if err != nil {
		return err
	}

	fullPath := filepath.Join(conf.TestRoot, path)

	runConfig := run.RunConfig{
		Workers:     conf.Workers,
		Skips:       true,
		Timings:     false,
		Timeout:     30 * time.Second,
		Interactive: false,
	}

	newResults, _ := run.TestsInDir(fullPath, runConfig)

	existingResults, err := results.LoadResults()
	if err == nil && existingResults != nil {
		merged := newResults.MergeInto(existingResults)
		merged.Write()
	} else {
		newResults.Write()
	}

	return current(c)
}

func info(c *fiber.Ctx) error {
	res, err := cache.GetResultsIndex()
	if err != nil {
		return err
	}

	path, err := filepath.Rel("/api/info/", c.Path())
	if err != nil {
		return err
	}

	path = strings.TrimSuffix(path, ".json")

	fullPath := filepath.Join(conf.TestRoot, path)

	if res, ok := (*res)[fullPath]; ok {
		return c.Status(fiber.StatusOK).JSON(res)
	}

	return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
		"error": "Test not found",
	})
}
