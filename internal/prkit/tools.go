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
	// We use exec.Command directly because Runner might not have RepoRoot
	// initialized yet, and strict Runner now requires RepoRoot to be set.
	cmd := exec.Command("git", "rev-parse", "--show-toplevel")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("failed to find repo root: %v, output: %s", err, string(out))
	}
	return strings.TrimSpace(string(out)), nil
}
