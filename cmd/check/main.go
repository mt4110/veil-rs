package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
	"veil-rs/internal/ux"
)

type CheckMetric struct {
	MetricsVersion string `json:"metrics_version"`
	Command        string `json:"command"`
	Result         string `json:"result"`
	Timestamp      string `json:"timestamp"`
	Git            struct {
		SHA   string `json:"sha"`
		Dirty bool   `json:"dirty"`
	} `json:"git"`
	Reasons []MetricReason `json:"reasons"`
}

type MetricReason struct {
	Code   string `json:"code"`
	Reason string `json:"reason"`
	Fix    string `json:"fix"`
	Docs   string `json:"docs"`
}

func main() {
	// 1. Setup UX Output
	out := ux.Output{
		Step:   "check",
		Status: "PASS",
	}

	// TODO: Run actual checks here
	// For now, we simulate a PASS or conditional FAIL based on env for testing
	if os.Getenv("FORCE_FAIL") == "1" {
		out.Add("FORCE_FAIL", "Forced failure for testing", "Unset FORCE_FAIL", "docs/ai/SSOT.md#testing")
	}

	// 2. Print for Human/Machine (stdout)
	out.Print()

	// 3. Write Metrics (best effort)
	writeMetrics(&out)

	if out.Status == "FAIL" {
		os.Exit(1)
	}
}

func writeMetrics(out *ux.Output) {
	metric := CheckMetric{
		MetricsVersion: "v1",
		Command:        "check",
		Result:         "pass",
		Timestamp:      time.Now().UTC().Format(time.RFC3339),
	}
	if out.Status == "FAIL" {
		metric.Result = "fail"
	}
	
	// Get real Git info
	metric.Git.SHA = getGitSHA()
	metric.Git.Dirty = isGitDirty()

	for _, r := range out.Reasons {
		// Find corresponding fix and doc (simplified matching for prototype)
		fix := ""
		doc := ""
		for _, f := range out.Fixes {
			if f.Code == r.Code {
				fix = f.Message
				break
			}
		}
		for _, d := range out.Docs {
			if d.Code == r.Code {
				doc = d.Reference
				break
			}
		}

		metric.Reasons = append(metric.Reasons, MetricReason{
			Code:   r.Code,
			Reason: r.Message,
			Fix:    fix,
			Docs:   doc,
		})
	}

	data, err := json.MarshalIndent(metric, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling metrics: %v\n", err)
		return
	}
	
	dir := "dist/metrics/v1"
	if err := os.MkdirAll(dir, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "Error creating metrics dir: %v\n", err)
		return
	}
	
	file := filepath.Join(dir, "check.json")
	if err := os.WriteFile(file, data, 0644); err != nil {
		fmt.Fprintf(os.Stderr, "Error writing metrics: %v\n", err)
	}
}

func getGitSHA() string {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	out, err := cmd.Output()
	if err != nil {
		return "unknown"
	}
	return strings.TrimSpace(string(out))
}

func isGitDirty() bool {
	cmd := exec.Command("git", "status", "--porcelain")
	out, err := cmd.Output()
	if err != nil {
		return false // Assume clean if git fails, or handle error? For now, fail safe.
	}
	return len(strings.TrimSpace(string(out))) > 0
}
