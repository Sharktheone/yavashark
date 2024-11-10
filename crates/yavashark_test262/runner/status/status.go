package status

import "encoding/json"

type Status uint8

const (
	PASS Status = iota
	FAIL
	SKIP
	TIMEOUT
	CRASH
	PARSE_ERROR
	NOT_IMPLEMENTED
	RUNNER_ERROR
)

func (s Status) MarshalJSON() ([]byte, error) {
	var statusStr string
	switch s {
	case PASS:
		statusStr = "PASS"
	case FAIL:
		statusStr = "FAIL"
	case SKIP:
		statusStr = "SKIP"
	case TIMEOUT:
		statusStr = "TIMEOUT"
	case CRASH:
		statusStr = "CRASH"
	case PARSE_ERROR:
		statusStr = "PARSE_ERROR"
	case NOT_IMPLEMENTED:
		statusStr = "NOT_IMPLEMENTED"
	case RUNNER_ERROR:
		statusStr = "RUNNER_ERROR"
	default:
		statusStr = "UNKNOWN"
	}
	return json.Marshal(statusStr)
}
