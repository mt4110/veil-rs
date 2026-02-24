package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestS12_07_DocsPrNaming(t *testing.T) {
	td := t.TempDir()

	must := func(err error) {
		if err != nil {
			t.Fatalf("setup failed: %v", err)
		}
	}
	must(os.MkdirAll(filepath.Join(td, "docs", "pr"), 0o755))
	must(os.MkdirAll(filepath.Join(td, "docs", "ops"), 0o755))

	must(os.WriteFile(filepath.Join(td, "docs", "pr", "PR-91-s12-07-ok.md"), []byte("OK: content\n"), 0o644))
	must(os.WriteFile(filepath.Join(td, "docs", "pr", "PR-TBD-s12-07-bad.md"), []byte("OK: content\n"), 0o644))

	status := "| S12-07 | x | 1% | y | docs/pr/PR-9999-missing.md |\n"
	must(os.WriteFile(filepath.Join(td, "docs", "ops", "STATUS.md"), []byte(status), 0o644))

	cwd, _ := os.Getwd()
	_ = os.Chdir(td)
	defer func() { _ = os.Chdir(cwd) }()

	if got := s12_07_scan_docs_pr("."); got == 0 {
		t.Fatalf("expected stop=1, got stop=0")
	}
}

func TestS12_07_PythonStdoutAudit(t *testing.T) {
	td := t.TempDir()

	must := func(err error) {
		if err != nil {
			t.Fatalf("setup failed: %v", err)
		}
	}
	must(os.MkdirAll(filepath.Join(td, "scripts"), 0o755))

	py := "if __name__ == \"__main__\":\n    print(\"hi\")\n"
	must(os.WriteFile(filepath.Join(td, "scripts", "entry.py"), []byte(py), 0o644))

	cwd, _ := os.Getwd()
	_ = os.Chdir(td)
	defer func() { _ = os.Chdir(cwd) }()

	if got := s12_07_scan_stdout_audit("."); got == 0 {
		t.Fatalf("expected stop=1, got stop=0")
	}
}
