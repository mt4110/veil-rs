package cirepro

import (
	"fmt"
	"strings"
)

// renderSummary produces the fixed-format markdown summary per PLAN.
func renderSummary(tsUTC, runID string, gi GitInfo, outDir string, argv []string, steps []StepResult, overall, summaryPath, snapshotPath string) string {
	cmdLine := strings.Join(argv, " ")
	if strings.TrimSpace(cmdLine) == "" {
		cmdLine = "-"
	}

	b := &strings.Builder{}
	b.WriteString("# ci-repro summary\n\n")

	b.WriteString("## Run\n")
	fmt.Fprintf(b, "- run_id: %s\n", runID)
	fmt.Fprintf(b, "- timestamp_utc: %s\n", tsUTC)
	fmt.Fprintf(b, "- git_sha: %s\n", nonEmpty(gi.SHA, nonEmpty(gi.SHA7, "UNKNOWN")))
	fmt.Fprintf(b, "- git_tree: %s\n", nonEmpty(gi.TreeStatus, "UNKNOWN"))
	fmt.Fprintf(b, "- out_dir: %s\n", outDir)
	fmt.Fprintf(b, "- command: %s\n\n", cmdLine)

	b.WriteString("## Steps\n")
	b.WriteString("| idx | step          | status | started_utc | ended_utc | duration_ms | log_file |\n")
	b.WriteString("| --- | ------------- | ------ | ----------- | --------- | ----------- | -------- |\n")
	for _, r := range steps {
		dur := "-"
		if r.DurationMs >= 0 && r.StartedUTC != "-" {
			dur = fmt.Sprintf("%d", r.DurationMs)
		}
		fmt.Fprintf(b, "| %s  | %-13s | %-6s | %s | %s | %s | %s |\n",
			pad2(r.Index), r.Name, r.Status, r.StartedUTC, r.EndedUTC, dur, r.LogFile)
	}
	b.WriteString("\n")

	// error_steps
	var errs []string
	for _, r := range steps {
		if r.Status == "ERROR" {
			errs = append(errs, r.Index)
		}
	}
	errSteps := "NONE"
	if len(errs) > 0 {
		errSteps = strings.Join(errs, ",")
	}

	b.WriteString("## Final\n")
	fmt.Fprintf(b, "- overall: %s\n", overall)
	fmt.Fprintf(b, "- error_steps: %s\n\n", errSteps)

	b.WriteString("## Files\n")
	fmt.Fprintf(b, "- summary: %s\n", summaryPath)
	fmt.Fprintf(b, "- status_snapshot: %s\n", snapshotPath)

	return b.String()
}

func nonEmpty(v, def string) string {
	if strings.TrimSpace(v) == "" {
		return def
	}
	return v
}

func pad2(s string) string {
	if len(s) == 1 {
		return "0" + s
	}
	return s
}
