package scripting

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sync"
	"time"
	"viewer/conf"
)

// Runtime manages the Deno subprocess for executing TypeScript scripts
type Runtime struct {
	cmd      *exec.Cmd
	stdin    io.WriteCloser
	stdout   *bufio.Reader
	stderr   io.ReadCloser
	mu       sync.Mutex
	running  bool
	scriptID int64
}

// ScriptResult represents the result of executing a script
type ScriptResult struct {
	Success bool            `json:"success"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   string          `json:"error,omitempty"`
}

// NewRuntime creates a new Deno runtime instance
func NewRuntime() *Runtime {
	return &Runtime{}
}

// Start launches the Deno subprocess
func (r *Runtime) Start() error {
	r.mu.Lock()
	defer r.mu.Unlock()

	if r.running {
		return nil
	}

	// Find the runtime script path
	runtimeScript, err := r.findRuntimeScript()
	if err != nil {
		return fmt.Errorf("failed to find runtime script: %w", err)
	}

	// Build Deno command
	r.cmd = exec.Command(conf.DenoPath, "run",
		"--allow-net=localhost",
		"--allow-read",
		"--no-prompt",
		runtimeScript,
	)

	// Set up pipes
	r.stdin, err = r.cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdin pipe: %w", err)
	}

	stdout, err := r.cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdout pipe: %w", err)
	}
	r.stdout = bufio.NewReader(stdout)

	r.stderr, err = r.cmd.StderrPipe()
	if err != nil {
		return fmt.Errorf("failed to create stderr pipe: %w", err)
	}

	// Start the process
	if err := r.cmd.Start(); err != nil {
		return fmt.Errorf("failed to start Deno: %w", err)
	}

	r.running = true

	// Start stderr reader goroutine
	go r.readStderr()

	return nil
}

// Stop terminates the Deno subprocess
func (r *Runtime) Stop() error {
	r.mu.Lock()
	defer r.mu.Unlock()

	if !r.running {
		return nil
	}

	r.running = false

	// Close stdin to signal shutdown
	if r.stdin != nil {
		r.stdin.Close()
	}

	// Give process time to exit gracefully
	done := make(chan error, 1)
	go func() {
		done <- r.cmd.Wait()
	}()

	select {
	case <-done:
		// Process exited
	case <-time.After(2 * time.Second):
		// Force kill
		r.cmd.Process.Kill()
	}

	return nil
}

// Execute runs a TypeScript script and returns the result
func (r *Runtime) Execute(ctx context.Context, script string, sessionID string, serverURL string) (*ScriptResult, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	if !r.running {
		return nil, fmt.Errorf("runtime not started")
	}

	// Increment script ID for tracking
	r.scriptID++
	id := r.scriptID

	// Create request
	request := NewRPCRequest(id, "execute", ExecuteParams{
		Script:    script,
		SessionID: sessionID,
		Timeout:   conf.ScriptTimeout,
		ServerURL: serverURL,
	})

	// Send request
	reqBytes, err := json.Marshal(request)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	if _, err := r.stdin.Write(append(reqBytes, '\n')); err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}

	// Read response with context timeout
	responseCh := make(chan []byte, 1)
	errCh := make(chan error, 1)

	go func() {
		line, err := r.stdout.ReadBytes('\n')
		if err != nil {
			errCh <- err
			return
		}
		responseCh <- line
	}()

	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case err := <-errCh:
		return nil, fmt.Errorf("failed to read response: %w", err)
	case line := <-responseCh:
		var response RPCResponse
		if err := json.Unmarshal(line, &response); err != nil {
			return nil, fmt.Errorf("failed to unmarshal response: %w", err)
		}

		if response.Error != nil {
			return &ScriptResult{
				Success: false,
				Error:   response.Error.Message,
			}, nil
		}

		return &ScriptResult{
			Success: true,
			Result:  response.Result,
		}, nil
	}
}

// findRuntimeScript locates the TypeScript runtime script
func (r *Runtime) findRuntimeScript() (string, error) {
	// Check relative to the viewer binary
	execPath, err := os.Executable()
	if err == nil {
		execDir := filepath.Dir(execPath)
		candidates := []string{
			filepath.Join(execDir, "scripting", "runtime.ts"),
			filepath.Join(execDir, "..", "scripting", "runtime.ts"),
		}
		for _, path := range candidates {
			if _, err := os.Stat(path); err == nil {
				return path, nil
			}
		}
	}

	// Check relative to working directory
	wd, err := os.Getwd()
	if err == nil {
		candidates := []string{
			filepath.Join(wd, "scripting", "runtime.ts"),
			filepath.Join(wd, "viewer", "scripting", "runtime.ts"),
		}
		for _, path := range candidates {
			if _, err := os.Stat(path); err == nil {
				return path, nil
			}
		}
	}

	return "", fmt.Errorf("runtime.ts not found")
}

// readStderr reads and logs stderr output from Deno
func (r *Runtime) readStderr() {
	scanner := bufio.NewScanner(r.stderr)
	for scanner.Scan() {
		fmt.Fprintf(os.Stderr, "[deno] %s\n", scanner.Text())
	}
}

// IsRunning returns whether the runtime is currently active
func (r *Runtime) IsRunning() bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.running
}
