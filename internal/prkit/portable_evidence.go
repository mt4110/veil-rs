package prkit

import (
	"encoding/json"
	"fmt"
	"os"
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
	Name string `json:"name"`
	Cmd  string `json:"cmd"`
}

func (e *Evidence) PrintJSON() error {
	b, err := json.MarshalIndent(e, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(b))
	return nil
}

func (e *Evidence) WriteJSON(path string) error {
	b, err := json.MarshalIndent(e, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, append(b, '\n'), 0644)
}
