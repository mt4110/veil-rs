package cirepro

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Config holds ci-repro run configuration.
type Config struct {
	OutDir     string
	RunID      string
	WithStrict bool
	Command    []string
}

// Result holds the overall ci-repro outcome.
type Result struct {
	Overall            string // OK or ERROR
	SummaryPath        string
	StatusSnapshotPath string
	StepResults        []StepResult
	Notes              []string
}

// Deps provides injected dependencies to avoid side-effects in core logic.
type Deps struct {
	NowUTC          func() time.Time
	Getenv          func(key string) string
	MkdirAll        func(path string, perm os.FileMode) error
	WriteFileAtomic func(filename string, data []byte, perm os.FileMode) error
	ProbeGit        func() GitInfo
	RunToLog        func(argv []string, logFile string) (int, error)
}

// Run executes all steps in staged order (stopless).
func Run(cfg Config, deps Deps, stdout, stderr io.Writer) Result {
	return run(cfg, deps, "", stdout, stderr)
}

// RunStep executes a single named step (stopless).
func RunStep(cfg Config, deps Deps, stepName string, stdout, stderr io.Writer) Result {
	return run(cfg, deps, stepName, stdout, stderr)
}

func run(cfg Config, deps Deps, onlyStep string, stdout, stderr io.Writer) Result {
	outDir := strings.TrimSpace(cfg.OutDir)
	if outDir == "" {
		outDir = ".local/obs"
	}
	_ = deps.MkdirAll(outDir, 0o755)

	ts := deps.NowUTC().Format("20060102T150405Z")
	gi := deps.ProbeGit()

	runID := strings.TrimSpace(cfg.RunID)
	if runID == "" {
		sha7 := gi.SHA7
		if sha7 == "" {
			sha7 = "UNKNOWN"
		}
		runID = fmt.Sprintf("%s_%s", ts, sha7)
	}

	prefix := fmt.Sprintf("ci_%s", runID)
	summaryPath := filepath.Join(outDir, prefix+"_summary.md")
	snapshotPath := filepath.Join(outDir, prefix+"_status_snapshot.txt")

	steps := canonicalSteps(outDir, prefix)

	// Select steps to run
	selected := steps
	if onlyStep != "" {
		var tmp []StepDef
		for _, s := range steps {
			if s.Name == onlyStep {
				tmp = append(tmp, s)
			}
		}
		selected = tmp
	}

	res := Result{
		Overall:            "OK",
		SummaryPath:        summaryPath,
		StatusSnapshotPath: snapshotPath,
	}

	// Always write STATUS snapshot (even if git probe failed)
	snap := buildStatusSnapshot(ts, runID, gi, gi.RepoRoot)
	_ = deps.WriteFileAtomic(snapshotPath, []byte(snap), 0o644)

	prevError := false

	for _, step := range selected {
		// In "run" mode: strict steps require --with-strict
		if onlyStep == "" && isStrictStep(step.Name) && !cfg.WithStrict {
			r := stepSkipped(deps, step, "SKIP: strict steps require --with-strict")
			res.StepResults = append(res.StepResults, r)
			continue
		}

		// Blocked by previous ERROR
		if prevError {
			r := stepSkipped(deps, step, "SKIP: blocked by previous ERROR")
			res.StepResults = append(res.StepResults, r)
			continue
		}

		// DIRTY tree => skip prverify & strict (go-test is OK)
		if gi.TreeStatus == "DIRTY" && (step.Name == "prverify" || isStrictStep(step.Name)) {
			r := stepSkipped(deps, step, "SKIP: git_tree=DIRTY blocks prverify/strict")
			res.StepResults = append(res.StepResults, r)
			continue
		}

		// Execute step
		started := deps.NowUTC()
		code, runErr := deps.RunToLog(step.CmdArgv, step.LogFile)
		ended := deps.NowUTC()

		r := StepResult{
			Index:      step.Index,
			Name:       step.Name,
			Status:     "OK",
			StartedUTC: started.Format(time.RFC3339),
			EndedUTC:   ended.Format(time.RFC3339),
			DurationMs: ended.Sub(started).Milliseconds(),
			LogFile:    step.LogFile,
		}

		if runErr != nil || code != 0 {
			r.Status = "ERROR"
			r.Reason = fmt.Sprintf("ERROR: step failed (exit=%d)", code)
			prevError = true
			res.Overall = "ERROR"
		}

		res.StepResults = append(res.StepResults, r)
	}

	// Normalize to 4-row table
	full := normalizeToFourRows(steps, res.StepResults)

	// Write summary (always, even on failure)
	sum := renderSummary(ts, runID, gi, outDir, cfg.Command, full, res.Overall, summaryPath, snapshotPath)
	_ = deps.WriteFileAtomic(summaryPath, []byte(sum), 0o644)

	return Result{
		Overall:            res.Overall,
		SummaryPath:        summaryPath,
		StatusSnapshotPath: snapshotPath,
		StepResults:        full,
		Notes:              res.Notes,
	}
}

func isStrictStep(name string) bool {
	return name == "strict-create" || name == "strict-verify"
}

func stepSkipped(deps Deps, step StepDef, reason string) StepResult {
	_ = deps.WriteFileAtomic(step.LogFile, []byte(reason+"\ncmd: "+strings.Join(step.CmdArgv, " ")+"\n"), 0o644)
	return StepResult{
		Index: step.Index, Name: step.Name, Status: "SKIP",
		StartedUTC: "-", EndedUTC: "-", DurationMs: -1,
		LogFile: step.LogFile, Reason: reason,
	}
}
