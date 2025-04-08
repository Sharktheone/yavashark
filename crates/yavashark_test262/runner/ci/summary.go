package ci

type Summary struct {
	Passed         uint32 `json:"passed"`
	Failed         uint32 `json:"failed"`
	Skipped        uint32 `json:"skipped"`
	NotImplemented uint32 `json:"not_implemented"`
	RunnerError    uint32 `json:"runner_error"`
	Crashed        uint32 `json:"crashed"`
	Timeout        uint32 `json:"timeout"`
	ParseError     uint32 `json:"parse_error"`
	Total          uint32 `json:"total"`
	Timestamp      int64  `json:"time"`
	CommitHash     string `json:"commit_hash"`
}

type History struct {
	Runs []Summary `json:"runs"`
}

type DirectorySummary struct {
	Directory      string `json:"directory"`
	Passed         int    `json:"passed"`
	Failed         int    `json:"failed"`
	Skipped        int    `json:"skipped"`
	NotImplemented int    `json:"not_implemented"`
	RunnerError    int    `json:"runner_error"`
	Crashed        int    `json:"crashed"`
	Timeout        int    `json:"timeout"`
	ParseError     int    `json:"parse_error"`
	Total          int    `json:"total"`
}
