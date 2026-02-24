package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestLocalgcStopless verifies that run() always exits 0 and emits
// OK: phase=end stop=<0|1> (S12-06C stopless invariant).
func TestLocalgcStopless(t *testing.T) {
	cases := []struct {
		name     string
		args     []string
		wantStop string
	}{
		{"dry-run default", []string{"--root", t.TempDir()}, "stop=0"},
		{"plan mode", []string{"--mode", "plan", "--root", t.TempDir()}, "stop=0"},
		{"bad flag", []string{"--bad-flag-xyz"}, "stop=1"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			rc := run(tc.args, &stdout, &stderr)
			if rc != 0 {
				t.Errorf("expected rc=0 (stopless), got %d", rc)
			}
			out := stdout.String()
			if !strings.Contains(out, "OK: phase=end") {
				t.Errorf("missing OK: phase=end in stdout:\n%s", out)
			}
			if !strings.Contains(out, tc.wantStop) {
				t.Errorf("expected %q in stdout:\n%s", tc.wantStop, out)
			}
		})
	}
}

// TestLocalgcDoubleLock verifies the double-lock requirement for apply mode.
// --mode apply without --apply → SKIP
// --apply without --mode apply → SKIP
// Both together → proceeds (no double-lock SKIP)
func TestLocalgcDoubleLock(t *testing.T) {
	t.Run("mode_apply_without_apply_flag", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		rc := run([]string{"--mode", "apply", "--root", t.TempDir()}, &stdout, &stderr)
		if rc != 0 {
			t.Errorf("expected rc=0 (stopless), got %d", rc)
		}
		out := stdout.String()
		if !strings.Contains(out, "SKIP:") {
			t.Errorf("expected SKIP: in stdout (missing --apply):\n%s", out)
		}
		if !strings.Contains(out, "stop=0") {
			t.Errorf("expected stop=0 (clean skip):\n%s", out)
		}
	})

	t.Run("apply_flag_without_mode_apply", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		// --apply alone (mode defaults to dry-run)
		rc := run([]string{"--apply", "--root", t.TempDir()}, &stdout, &stderr)
		if rc != 0 {
			t.Errorf("expected rc=0 (stopless), got %d", rc)
		}
		out := stdout.String()
		if !strings.Contains(out, "SKIP:") {
			t.Errorf("expected SKIP: in stdout (missing --mode apply):\n%s", out)
		}
		if !strings.Contains(out, "stop=0") {
			t.Errorf("expected stop=0 (clean skip):\n%s", out)
		}
	})

	t.Run("both_flags_proceeds", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		root := t.TempDir()
		rc := run([]string{"--mode", "apply", "--apply", "--root", root}, &stdout, &stderr)
		if rc != 0 {
			t.Errorf("expected rc=0 (stopless), got %d", rc)
		}
		out := stdout.String()
		// Should NOT produce double-lock SKIP
		if strings.Contains(out, "SKIP: --mode apply requires") {
			t.Errorf("should not SKIP when both flags present:\n%s", out)
		}
		if strings.Contains(out, "SKIP: --apply requires") {
			t.Errorf("should not SKIP when both flags present:\n%s", out)
		}
		if !strings.Contains(out, "OK: phase=end stop=0") {
			t.Errorf("expected OK: phase=end stop=0:\n%s", out)
		}
	})
}

// TestLocalgcDryRunSafe verifies that dry-run mode never removes files
// even when there are more entries than the retention limit.
func TestLocalgcDryRunSafe(t *testing.T) {
	root := t.TempDir()
	// Create a fake .local/prverify dir with 10 entries (exceeds retainPrverify=5)
	localPrverify := filepath.Join(root, ".local", "prverify")
	if err := os.MkdirAll(localPrverify, 0755); err != nil {
		t.Fatal(err)
	}
	for i := 0; i < 10; i++ {
		name := filepath.Join(localPrverify, "prverify_file"+string(rune('A'+i))+".md")
		if err := os.WriteFile(name, []byte("content"), 0644); err != nil {
			t.Fatal(err)
		}
	}

	var stdout, stderr bytes.Buffer
	rc := run([]string{"--mode", "dry-run", "--root", root}, &stdout, &stderr)
	if rc != 0 {
		t.Errorf("expected rc=0 (stopless), got %d", rc)
	}

	// All 10 files must still exist after dry-run
	entries, err := readEntries(localPrverify)
	if err != nil {
		t.Fatal(err)
	}
	if len(entries) != 10 {
		out := stdout.String()
		t.Errorf("dry-run removed files: expected 10, got %d\nstdout:\n%s", len(entries), out)
	}
}

// TestLocalgcOutputFormat verifies the OK: dir=... count=... format
// required for machine parsing.
func TestLocalgcOutputFormat(t *testing.T) {
	root := t.TempDir()
	var stdout, stderr bytes.Buffer
	rc := run([]string{"--root", root}, &stdout, &stderr)
	if rc != 0 {
		t.Errorf("expected rc=0, got %d", rc)
	}
	out := stdout.String()
	// Each category should produce OK: dir=<label> count=<N> ...
	for _, label := range []string{"prverify", "review-bundles", "obs"} {
		if !strings.Contains(out, "OK: dir="+label) {
			t.Errorf("missing OK: dir=%s in stdout:\n%s", label, out)
		}
	}
}
