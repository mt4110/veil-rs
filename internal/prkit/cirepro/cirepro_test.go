package cirepro

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func defaultTestDeps(gi GitInfo, runToLog func(argv []string, logFile string) (int, error)) Deps {
	return Deps{
		NowUTC:          func() time.Time { return time.Now().UTC() },
		Getenv:          os.Getenv,
		MkdirAll:        os.MkdirAll,
		WriteFileAtomic: WriteFileAtomic,
		ProbeGit:        func() GitInfo { return gi },
		RunToLog:        runToLog,
	}
}

func TestRunIDFixedProducesFixedFilenames(t *testing.T) {
	tmp := t.TempDir()

	gi := GitInfo{SHA: "deadbeef", SHA7: "deadbee", TreeStatus: "CLEAN"}
	runToLog := func(argv []string, logFile string) (int, error) {
		_ = WriteFileAtomic(logFile, []byte("OK: fake\n"), 0o644)
		return 0, nil
	}
	deps := defaultTestDeps(gi, runToLog)

	cfg := Config{OutDir: tmp, RunID: "fixed", WithStrict: false, Command: []string{"ci-repro", "run", "--run-id", "fixed"}}
	res := Run(cfg, deps, os.Stdout, os.Stderr)

	if !strings.Contains(filepath.Base(res.SummaryPath), "ci_fixed_summary.md") {
		t.Fatalf("summary path not fixed: %s", res.SummaryPath)
	}
	if _, err := os.Stat(res.SummaryPath); err != nil {
		t.Fatalf("missing summary: %v", err)
	}
	if _, err := os.Stat(res.StatusSnapshotPath); err != nil {
		t.Fatalf("missing snapshot: %v", err)
	}
}

func TestDirtyTreeSkipsPrverifyAndStrict(t *testing.T) {
	tmp := t.TempDir()

	gi := GitInfo{SHA: "deadbeef", SHA7: "deadbee", TreeStatus: "DIRTY"}
	calls := 0
	runToLog := func(argv []string, logFile string) (int, error) {
		calls++
		_ = WriteFileAtomic(logFile, []byte("OK: fake\n"), 0o644)
		return 0, nil
	}
	deps := defaultTestDeps(gi, runToLog)

	cfg := Config{OutDir: tmp, RunID: "fixed", WithStrict: true}
	res := Run(cfg, deps, os.Stdout, os.Stderr)

	// go-test should run; prverify/strict should SKIP
	if calls != 1 {
		t.Fatalf("expected only go-test to run (calls=1), got %d", calls)
	}
	for _, r := range res.StepResults {
		if r.Name == "prverify" && r.Status != "SKIP" {
			t.Fatalf("prverify should SKIP on DIRTY, got %s", r.Status)
		}
		if (r.Name == "strict-create" || r.Name == "strict-verify") && r.Status != "SKIP" {
			t.Fatalf("strict should SKIP on DIRTY, got %s for %s", r.Status, r.Name)
		}
	}
}

func TestSkipWritesLogWithReason(t *testing.T) {
	tmp := t.TempDir()

	gi := GitInfo{SHA: "abc", SHA7: "abc1234", TreeStatus: "CLEAN"}
	runToLog := func(argv []string, logFile string) (int, error) {
		_ = WriteFileAtomic(logFile, []byte("OK: fake\n"), 0o644)
		return 0, nil
	}
	deps := defaultTestDeps(gi, runToLog)

	// Run without --with-strict => strict steps get SKIP log
	cfg := Config{OutDir: tmp, RunID: "fixed", WithStrict: false}
	res := Run(cfg, deps, os.Stdout, os.Stderr)

	for _, r := range res.StepResults {
		if r.Name == "strict-create" {
			if r.Status != "SKIP" {
				t.Fatalf("strict-create should SKIP, got %s", r.Status)
			}
			// Check log file exists and contains SKIP
			data, err := os.ReadFile(r.LogFile)
			if err != nil {
				t.Fatalf("cannot read skip log: %v", err)
			}
			if !strings.Contains(string(data), "SKIP:") {
				t.Fatalf("skip log missing SKIP reason: %s", string(data))
			}
			if !strings.Contains(string(data), "cmd:") {
				t.Fatalf("skip log missing cmd header: %s", string(data))
			}
		}
	}
}

func TestRepoMissingSummaryStillWritten(t *testing.T) {
	tmp := t.TempDir()

	gi := GitInfo{TreeStatus: "UNKNOWN"} // no repo root
	runToLog := func(argv []string, logFile string) (int, error) {
		_ = WriteFileAtomic(logFile, []byte("OK: fake\n"), 0o644)
		return 0, nil
	}
	deps := defaultTestDeps(gi, runToLog)

	cfg := Config{OutDir: tmp, RunID: "fixed"}
	res := Run(cfg, deps, os.Stdout, os.Stderr)

	if _, err := os.Stat(res.SummaryPath); err != nil {
		t.Fatalf("summary missing when repo unknown: %v", err)
	}
	if _, err := os.Stat(res.StatusSnapshotPath); err != nil {
		t.Fatalf("snapshot missing when repo unknown: %v", err)
	}

	// Snapshot should contain ERROR about repo root
	data, _ := os.ReadFile(res.StatusSnapshotPath)
	if !strings.Contains(string(data), "ERROR:") {
		t.Fatalf("snapshot should contain ERROR when repo unknown: %s", string(data))
	}
}

func TestSummaryFormatStability(t *testing.T) {
	tmp := t.TempDir()

	gi := GitInfo{SHA: "deadbeef", SHA7: "deadbee", TreeStatus: "CLEAN"}
	runToLog := func(argv []string, logFile string) (int, error) {
		_ = WriteFileAtomic(logFile, []byte("OK: fake\n"), 0o644)
		return 0, nil
	}
	deps := defaultTestDeps(gi, runToLog)

	cfg := Config{OutDir: tmp, RunID: "fixed", Command: []string{"ci-repro", "run"}}
	res := Run(cfg, deps, os.Stdout, os.Stderr)

	data, err := os.ReadFile(res.SummaryPath)
	if err != nil {
		t.Fatalf("cannot read summary: %v", err)
	}
	content := string(data)

	for _, section := range []string{"# ci-repro summary", "## Run", "## Steps", "## Final", "## Files"} {
		if !strings.Contains(content, section) {
			t.Errorf("summary missing section: %s", section)
		}
	}
	// Must have 4 step rows (go-test, prverify, strict-create, strict-verify)
	for _, name := range []string{"go-test", "prverify", "strict-create", "strict-verify"} {
		if !strings.Contains(content, name) {
			t.Errorf("summary missing step: %s", name)
		}
	}
}
