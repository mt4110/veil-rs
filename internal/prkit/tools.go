package prkit

import (
	"context"
	"fmt"
	"os/exec"

	"strings"
)

func collectToolVersions() []ToolVersion {
	tools := []string{"go", "git", "rustc", "cargo", "nix"}
	var versions []ToolVersion

	for _, tool := range tools {
		v, err := getToolVersion(tool)
		if err != nil {
			versions = append(versions, ToolVersion{
				Name:    tool,
				Version: fmt.Sprintf("skip: %v", err), // 1-line reason
			})
		} else {
			versions = append(versions, ToolVersion{
				Name:    tool,
				Version: v,
			})
		}
	}
	return versions
}

func getToolVersion(tool string) (string, error) {
	_, err := exec.LookPath(tool)
	if err != nil {
		return "", fmt.Errorf("not found in PATH: %w", err)
	}

	// Most tools support --version, some like go use version
	arg := "--version"
	if tool == "go" {
		arg = "version"
	}

	// Using ExecRunner
	spec := ExecSpec{
		Argv: []string{tool, arg},
	}
	res := Runner.Run(context.Background(), spec)

	if res.ExitCode != 0 {
		return "", fmt.Errorf("execution failed: %s, stderr: %s", res.ErrorKind, strings.TrimSpace(res.Stderr))
	}

	return strings.TrimSpace(res.Stdout), nil
}

// FindRepoRoot returns the absolute path to the repository root.
func FindRepoRoot() (string, error) {
	// git rev-parse --show-toplevel
	spec := ExecSpec{
		Argv: []string{"git", "rev-parse", "--show-toplevel"},
	}
	// Note: We use the *current* executor state. If Init() hasn't been called,
	// RepoRoot is empty, so this runs in CWD. This is correct for bootstrapping.
	res := Runner.Run(context.Background(), spec)

	if res.ExitCode != 0 {
		return "", fmt.Errorf("failed to find repo root: %s", res.Stderr)
	}
	return strings.TrimSpace(res.Stdout), nil
}
