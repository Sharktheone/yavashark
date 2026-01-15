package scripting

import "encoding/json"

// JSON-RPC 2.0 protocol types for communication with Deno runtime

// RPCRequest represents a JSON-RPC request from Go to Deno
type RPCRequest struct {
	JSONRPC string `json:"jsonrpc"`
	ID      int64  `json:"id"`
	Method  string `json:"method"`
	Params  any    `json:"params,omitempty"`
}

// RPCResponse represents a JSON-RPC response from Deno to Go
type RPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      int64           `json:"id"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *RPCError       `json:"error,omitempty"`
}

// RPCError represents a JSON-RPC error
type RPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Data    any    `json:"data,omitempty"`
}

// ExecuteParams contains parameters for the "execute" RPC method
type ExecuteParams struct {
	Script    string `json:"script"`
	SessionID string `json:"sessionId,omitempty"`
	Timeout   int    `json:"timeout"`   // seconds
	ServerURL string `json:"serverUrl"` // URL for API calls back to Go
}

// API call parameters (Go -> Deno -> Go round-trip for ys.* calls)

// APICallRequest represents an API call from the script to the Go server
type APICallRequest struct {
	ID     int64  `json:"id"`
	Method string `json:"method"` // e.g., "tests.getStatus", "runner.rerun"
	Params any    `json:"params,omitempty"`
}

// APICallResponse represents the response to an API call
type APICallResponse struct {
	ID     int64           `json:"id"`
	Result json.RawMessage `json:"result,omitempty"`
	Error  *RPCError       `json:"error,omitempty"`
}

// Standard error codes
const (
	ErrCodeParse          = -32700
	ErrCodeInvalidRequest = -32600
	ErrCodeMethodNotFound = -32601
	ErrCodeInvalidParams  = -32602
	ErrCodeInternal       = -32603
	ErrCodeTimeout        = -32000
	ErrCodeScriptError    = -32001
)

// NewRPCRequest creates a new JSON-RPC request
func NewRPCRequest(id int64, method string, params any) *RPCRequest {
	return &RPCRequest{
		JSONRPC: "2.0",
		ID:      id,
		Method:  method,
		Params:  params,
	}
}

// NewRPCResponse creates a successful JSON-RPC response
func NewRPCResponse(id int64, result any) (*RPCResponse, error) {
	resultBytes, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}
	return &RPCResponse{
		JSONRPC: "2.0",
		ID:      id,
		Result:  resultBytes,
	}, nil
}

// NewRPCErrorResponse creates an error JSON-RPC response
func NewRPCErrorResponse(id int64, code int, message string) *RPCResponse {
	return &RPCResponse{
		JSONRPC: "2.0",
		ID:      id,
		Error: &RPCError{
			Code:    code,
			Message: message,
		},
	}
}
