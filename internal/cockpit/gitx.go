package cockpit

import (
	"bytes"
	"fmt"
	"os/exec"
	"strings"
)

type GitX struct {
	Dir string // optional; empty means current working directory
}

func (g GitX) Run(args ...string) (string, error) {
	if len(args) == 0 {
		return "", fmt.Errorf("gitx: no args")
	}

	cmd := exec.Command("git", args...)
	if g.Dir != "" {
		cmd.Dir = g.Dir
	}

	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		// keep stderr; include exit-ish context
		msg := strings.TrimSpace(stderr.String())
		if msg == "" {
			msg = err.Error()
		}
		return "", fmt.Errorf("git %s failed: %s", strings.Join(args, " "), msg)
	}

	return stdout.String(), nil
}
