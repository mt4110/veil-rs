package prkit

import (
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
				Version: fmt.Sprintf("skip %s: %v", tool, err), // 1-line reason
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
		return "", fmt.Errorf("not found in PATH")
	}

	// Most tools support --version, some like go use version
	arg := "--version"
	if tool == "go" {
		arg = "version"
	}

	cmd := exec.Command(tool, arg)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("execution failed: %v, output: %s", err, strings.TrimSpace(string(out)))
	}

	return strings.TrimSpace(string(out)), nil
}
