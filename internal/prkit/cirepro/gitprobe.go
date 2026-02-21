package cirepro

import (
	"os/exec"
	"strings"
)

// GitInfo holds the result of probing git state.
type GitInfo struct {
	RepoRoot   string
	SHA        string
	SHA7       string
	TreeStatus string // CLEAN, DIRTY, or UNKNOWN
}

// probeGit probes git state via exec.Command.
// Never panics or exits. Tests swap probeGitFn in cirepro.go.
func probeGit() GitInfo {
	gi := GitInfo{TreeStatus: "UNKNOWN"}

	if _, err := exec.LookPath("git"); err != nil {
		return gi
	}

	if out, err := exec.Command("git", "rev-parse", "--show-toplevel").CombinedOutput(); err == nil {
		gi.RepoRoot = strings.TrimSpace(string(out))
	}

	if out, err := exec.Command("git", "rev-parse", "HEAD").CombinedOutput(); err == nil {
		gi.SHA = strings.TrimSpace(string(out))
	}

	if out, err := exec.Command("git", "rev-parse", "--short=7", "HEAD").CombinedOutput(); err == nil {
		gi.SHA7 = strings.TrimSpace(string(out))
	}

	if out, err := exec.Command("git", "status", "--porcelain").CombinedOutput(); err == nil {
		if strings.TrimSpace(string(out)) == "" {
			gi.TreeStatus = "CLEAN"
		} else {
			gi.TreeStatus = "DIRTY"
		}
	}

	return gi
}
