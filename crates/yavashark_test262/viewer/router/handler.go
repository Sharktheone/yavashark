package router

import (
	"github.com/gofiber/fiber/v2"
	"os"
)

func current(c *fiber.Ctx) error {
	f, err := os.Open("results.json")
	if err != nil {
		return err
	}

	defer f.Close()

	//TODO: we need to parse the results.jsn file and convert it to the ci format

	return nil
}

func rerunAll(c *fiber.Ctx) error {
	if err := rebuild(); err != nil {
		return err
	}

	return nil
}

func rerun(c *fiber.Ctx) error {
	if err := rebuild(); err != nil {
		return err
	}

	return nil
}

func info(c *fiber.Ctx) error {
	//TODO: we need to parse the results.json file and get the info about the specific test

	return nil
}
