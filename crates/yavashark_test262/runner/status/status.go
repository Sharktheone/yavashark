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

func (s Status) String() string {
	switch s {
	case PASS:
		return "PASS"
	case FAIL:
		return "FAIL"
	case SKIP:
		return "SKIP"
	case TIMEOUT:
		return "TIMEOUT"
	case CRASH:
		return "CRASH"
	case PARSE_ERROR:
		return "PARSE_ERROR"
	case NOT_IMPLEMENTED:
		return "NOT_IMPLEMENTED"
	case RUNNER_ERROR:
		return "RUNNER_ERROR"
	default:
		return "UNKNOWN"
	}
}

func (s Status) MarshalJSON() ([]byte, error) {
	return json.Marshal(s.String())
}

type CIStatus string

const (
	CI_FAIL                CIStatus = "F"
	CI_CRASH                        = "C"
	CI_ERROR                        = "E"
	CI_TIMEOUT                      = "T"
	CI_OK                           = "O"
	CI_PASS                         = "P"
	CI_SKIP                         = "S"
	CI_NOT_RUN                      = "N"
	CI_PRECONDITION_FAILED          = "PF"
)

func (s CIStatus) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(s))
}

func (s Status) ToCIStatus() CIStatus {
	switch s {
	case PASS:
		return CI_PASS
	case FAIL:
		return CI_FAIL
	case SKIP:
		return CI_SKIP
	case TIMEOUT:
		return CI_TIMEOUT
	case CRASH:
		return CI_CRASH
	case PARSE_ERROR:
		return CI_OK
	case NOT_IMPLEMENTED:
		return CI_PRECONDITION_FAILED
	case RUNNER_ERROR:
		return CI_NOT_RUN
	default:
		return CI_ERROR
	}
}
