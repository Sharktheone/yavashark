package router

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os/exec"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
)

const (
	DataRepoOwner = "Sharktheone"
	DataRepoName  = "yavashark-data"
	GitHubAPIBase = "https://api.github.com"
)

type GitCommit struct {
	Hash      string    `json:"hash"`
	ShortHash string    `json:"shortHash"`
	Subject   string    `json:"subject"`
	Author    string    `json:"author"`
	Date      time.Time `json:"date"`
}

type GitHubCommit struct {
	SHA    string `json:"sha"`
	Commit struct {
		Message string `json:"message"`
		Author  struct {
			Name string    `json:"name"`
			Date time.Time `json:"date"`
		} `json:"author"`
	} `json:"commit"`
}

type GitHubBranch struct {
	Name string `json:"name"`
}

func getDataRepoCommits(c *fiber.Ctx) error {
	branch := c.Query("branch", "main")
	limit := c.QueryInt("limit", 30)

	url := fmt.Sprintf("%s/repos/%s/%s/commits?sha=%s&per_page=%d",
		GitHubAPIBase, DataRepoOwner, DataRepoName, branch, limit)

	resp, err := http.Get(url)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to fetch commits from GitHub: " + err.Error(),
		})
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		body, _ := io.ReadAll(resp.Body)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": fmt.Sprintf("GitHub API error (%d): %s", resp.StatusCode, string(body)),
		})
	}

	var ghCommits []GitHubCommit
	if err := json.NewDecoder(resp.Body).Decode(&ghCommits); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to parse GitHub response: " + err.Error(),
		})
	}

	commits := make([]GitCommit, 0, len(ghCommits))
	for _, ghc := range ghCommits {
		subject := ghc.Commit.Message
		if idx := strings.Index(subject, "\n"); idx > 0 {
			subject = subject[:idx]
		}

		commits = append(commits, GitCommit{
			Hash:      ghc.SHA,
			ShortHash: ghc.SHA[:7],
			Subject:   subject,
			Author:    ghc.Commit.Author.Name,
			Date:      ghc.Commit.Author.Date,
		})
	}

	return c.JSON(commits)
}

func getDataRepoBranches(c *fiber.Ctx) error {
	url := fmt.Sprintf("%s/repos/%s/%s/branches?per_page=100",
		GitHubAPIBase, DataRepoOwner, DataRepoName)

	resp, err := http.Get(url)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to fetch branches from GitHub: " + err.Error(),
		})
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		body, _ := io.ReadAll(resp.Body)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": fmt.Sprintf("GitHub API error (%d): %s", resp.StatusCode, string(body)),
		})
	}

	var ghBranches []GitHubBranch
	if err := json.NewDecoder(resp.Body).Decode(&ghBranches); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to parse GitHub response: " + err.Error(),
		})
	}

	branches := make([]string, 0, len(ghBranches))
	for _, b := range ghBranches {
		branches = append(branches, b.Name)
	}

	return c.JSON(branches)
}

func getCurrentGitInfo(c *fiber.Ctx) error {
	hashCmd := exec.Command("git", "rev-parse", "HEAD")
	hashOutput, err := hashCmd.Output()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to get current commit: " + err.Error(),
		})
	}
	hash := strings.TrimSpace(string(hashOutput))

	branchCmd := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD")
	branchOutput, _ := branchCmd.Output()
	branch := strings.TrimSpace(string(branchOutput))

	shortCmd := exec.Command("git", "rev-parse", "--short", "HEAD")
	shortOutput, _ := shortCmd.Output()
	shortHash := strings.TrimSpace(string(shortOutput))

	subjectCmd := exec.Command("git", "log", "-1", "--format=%s")
	subjectOutput, _ := subjectCmd.Output()
	subject := strings.TrimSpace(string(subjectOutput))

	return c.JSON(fiber.Map{
		"hash":      hash,
		"shortHash": shortHash,
		"branch":    branch,
		"subject":   subject,
	})
}

func getResultsForCommit(c *fiber.Ctx) error {
	hash := c.Params("hash")
	if hash == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "Commit hash is required",
		})
	}

	url := fmt.Sprintf("https://raw.githubusercontent.com/%s/%s/%s/results.json",
		DataRepoOwner, DataRepoName, hash)

	resp, err := http.Get(url)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to fetch results from GitHub: " + err.Error(),
		})
	}
	defer resp.Body.Close()

	if resp.StatusCode == 404 {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "Results not found for this commit",
		})
	}

	if resp.StatusCode != 200 {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": fmt.Sprintf("GitHub error (%d)", resp.StatusCode),
		})
	}

	c.Set("Content-Type", "application/json")
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to read response: " + err.Error(),
		})
	}

	return c.Send(body)
}
