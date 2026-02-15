package prkit

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"time"
)

type Evidence struct {
	SchemaVersion  int           `json:"schema_version"`
	TimestampUTC   string        `json:"timestamp_utc"`
	Mode           string        `json:"mode"`
	Status         string        `json:"status"`
	ExitCode       int           `json:"exit_code"`
	GitSHA         string        `json:"git_sha"`
	ToolVersions   []ToolVersion `json:"tool_versions"`
	Checks         []CheckResult `json:"checks"`
	CommandList    []Command     `json:"command_list"`
	ArtifactHashes []string      `json:"artifact_hashes"`
}

type ToolVersion struct {
	Name    string `json:"name"`
	Version string `json:"version"` // version string or "skip <tool>: <reason>"
}

type CheckResult struct {
	Name    string `json:"name"`
	Status  string `json:"status"` // PASS or FAIL
	Details string `json:"details"`
}

type Command struct {
	// Key Execution Details
	Argv     []string `json:"argv"`
	CwdRel   string   `json:"cwd_rel"`
	EnvMode  string   `json:"env_mode"`         // "inherit" or "inherit+delta"
	EnvKV    []string `json:"env_kv,omitempty"` // Environment variable overrides
	EnvHash  string   `json:"env_hash"`         // Hash of environment variables for determinism
	ExitCode int      `json:"exit_code"`

	// Output
	Stdout          string `json:"stdout"`
	Stderr          string `json:"stderr"`
	TruncatedStdout bool   `json:"truncated_stdout,omitempty"`
	TruncatedStderr bool   `json:"truncated_stderr,omitempty"`

	// Error Details
	ErrorKind string `json:"error_kind,omitempty"`
}

var Now = time.Now

func (e *Evidence) PrintJSON(w io.Writer) error {
	b, err := json.MarshalIndent(e, "", "  ")
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(w, string(b))
	return err
}

func (e *Evidence) WriteJSON(path string) error {
	b, err := json.MarshalIndent(e, "", "  ")
	if err != nil {
		return err
	}
	// Append newline for POSIX compatibility
	return os.WriteFile(path, append(b, '\n'), 0644)
}
