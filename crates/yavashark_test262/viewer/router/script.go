package router

import (
	"encoding/json"
	"viewer/scripting"

	"github.com/gofiber/fiber/v2"
)

type ScriptAPICallRequest struct {
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}

type ScriptAPICallResponse struct {
	Result any    `json:"result,omitempty"`
	Error  string `json:"error,omitempty"`
}

func scriptAPICall(c *fiber.Ctx) error {
	var req ScriptAPICallRequest
	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(ScriptAPICallResponse{
			Error: "invalid request body: " + err.Error(),
		})
	}

	result, err := scripting.HandleAPICall(req.Method, req.Params)
	if err != nil {
		return c.Status(fiber.StatusOK).JSON(ScriptAPICallResponse{
			Error: err.Error(),
		})
	}

	return c.Status(fiber.StatusOK).JSON(ScriptAPICallResponse{
		Result: result,
	})
}
