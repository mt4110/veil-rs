package cirepro

import (
	"os/exec"
	"strings"
)

// StepDef defines a canonical CI repro step.
type StepDef struct {
	Index   string   // "01", "02", "03", "04"
	Name    string   // go-test, prverify, strict-create, strict-verify
	CmdArgv []string // pinned command
	LogFile string   // full path to log file
}

// StepResult captures the outcome of a step execution.
type StepResult struct {
	Index      string // "01"..."04"
	Name       string
	Status     string // OK, ERROR, SKIP, or "-"
	StartedUTC string // RFC3339 or "-"
	EndedUTC   string // RFC3339 or "-"
	DurationMs int64  // -1 if not executed
	LogFile    string
	Reason     string
}

// canonicalSteps returns the 4 fixed steps with log file paths.
func canonicalSteps(outDir, prefix string) []StepDef {
	return []StepDef{
		{Index: "01", Name: "go-test", CmdArgv: []string{"nix", "develop", "-c", "go", "test", "./..."}, LogFile: outDir + "/" + prefix + "_step_01_go_test.log"},
		{Index: "02", Name: "prverify", CmdArgv: []string{"nix", "run", ".#prverify"}, LogFile: outDir + "/" + prefix + "_step_02_prverify.log"},
		{Index: "03", Name: "strict-create", CmdArgv: []string{"nix", "develop", "-c", "go", "run", "./cmd/reviewbundle", "strict", "create"}, LogFile: outDir + "/" + prefix + "_step_03_strict_create.log"},
		{Index: "04", Name: "strict-verify", CmdArgv: []string{"nix", "develop", "-c", "go", "run", "./cmd/reviewbundle", "strict", "verify"}, LogFile: outDir + "/" + prefix + "_step_04_strict_verify.log"},
	}
}

// normalizeToFourRows ensures exactly 4 rows in fixed order.
func normalizeToFourRows(all []StepDef, got []StepResult) []StepResult {
	m := map[string]StepResult{}
	for _, r := range got {
		m[r.Name] = r
	}
	out := make([]StepResult, 0, len(all))
	for _, s := range all {
		if r, ok := m[s.Name]; ok {
			out = append(out, r)
		} else {
			out = append(out, StepResult{
				Index: s.Index, Name: s.Name, Status: "-",
				StartedUTC: "-", EndedUTC: "-", DurationMs: -1,
				LogFile: s.LogFile,
			})
		}
	}
	return out
}

// runCommandToLog executes argv and writes output to logFile.
// Tests swap runToLogFn to avoid calling this.
func runCommandToLog(argv []string, logFile string) (int, error) {
	if len(argv) == 0 {
		_ = writeFileAtomic(logFile, []byte("ERROR: empty argv\n"), 0o644)
		return 1, nil
	}
	if _, err := exec.LookPath(argv[0]); err != nil {
		_ = writeFileAtomic(logFile, []byte("ERROR: command not found: "+argv[0]+"\n"), 0o644)
		return 127, nil
	}
	cmd := exec.Command(argv[0], argv[1:]...)
	b := &strings.Builder{}
	b.WriteString("cmd: " + strings.Join(argv, " ") + "\n")
	out, runErr := cmd.CombinedOutput()
	b.Write(out)
	_ = writeFileAtomic(logFile, []byte(b.String()), 0o644)
	if runErr != nil {
		if ee, ok := runErr.(*exec.ExitError); ok {
			return ee.ExitCode(), nil
		}
		return 1, runErr
	}
	return 0, nil
}
