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
	Argv            []string `json:"argv"`
	CwdRel          string   `json:"cwd_rel,omitempty"`
	Env             []EnvKV  `json:"env,omitempty"`
	Stdout          string   `json:"stdout"`
	Stderr          string   `json:"stderr"`
	ExitCode        int      `json:"exit_code"`
	ErrorKind       string   `json:"error_kind,omitempty"`
	TruncatedStdout bool     `json:"truncated_stdout,omitempty"`
	TruncatedStderr bool     `json:"truncated_stderr,omitempty"`
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
	return os.WriteFile(path, append(b, '\n'), 0644)
}
