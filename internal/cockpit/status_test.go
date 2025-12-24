package cockpit

import (
	"os/exec"
	"strings"
	"testing"
)

func TestStatusHasHeader(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git not found")
	}

	out, err := Status()
	if err != nil {
		t.Fatalf("Status() error: %v", err)
	}

	// `git status -sb` normally starts with "## ..."
	// detached HEAD may still include "## HEAD ..."
	if !strings.Contains(out, "##") {
		t.Fatalf("expected status output to contain '##', got: %q", out)
	}
}
