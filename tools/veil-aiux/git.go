package main

import (
	"bytes"
	"fmt"
	"os/exec"
	"sort"
	"strings"
)

// ---- Git helpers ----

type gitNumStat struct {
	add int
	del int
	n   int // file count
	nA  bool
}

func haveGit() bool {
	_, err := exec.LookPath("git")
	return err == nil
}

func gitCmd(args ...string) ([]byte, error) {
	cmd := exec.Command("git", args...)
	cmd.Stderr = new(bytes.Buffer)
	out, err := cmd.Output()
	if err != nil {
		// include stderr to make Fix actionable
		return nil, fmt.Errorf("git %s failed: %w (%s)", strings.Join(args, " "), err, strings.TrimSpace(cmd.Stderr.(*bytes.Buffer).String()))
	}
	return out, nil
}

func gitHeadSha() string {
	b, err := gitCmd("rev-parse", "HEAD")
	if err != nil {
		return "n/a"
	}
	return strings.TrimSpace(string(b))
}

func gitBaseExists(baseRef string) bool {
	_, err := gitCmd("rev-parse", "--verify", baseRef)
	return err == nil
}

func gitDiffNameStatus(baseRef string) ([]string, error) {
	b, err := gitCmd("diff", "--name-status", gitNoExtDiff, baseRef+gitDiffHeadSuffix)
	if err != nil {
		return nil, err
	}
	lines := strings.Split(strings.TrimSpace(string(b)), "\n")
	var out []string
	for _, ln := range lines {
		ln = strings.TrimSpace(ln)
		if ln == "" {
			continue
		}
		out = append(out, ln)
	}
	sort.Strings(out)
	if len(out) > maxAIPackFiles {
		out = append(out[:maxAIPackFiles], fmt.Sprintf("…TRUNCATED (%d+ more files)", len(out)-maxAIPackFiles))
	}
	return out, nil
}

func gitDiffNumStat(baseRef string) gitNumStat {
	if !gitBaseExists(baseRef) {
		return gitNumStat{nA: true}
	}
	b, err := gitCmd("diff", "--numstat", gitNoExtDiff, baseRef+gitDiffHeadSuffix)
	if err != nil {
		return gitNumStat{nA: true}
	}
	ns := gitNumStat{}
	for _, ln := range strings.Split(strings.TrimSpace(string(b)), "\n") {
		ln = strings.TrimSpace(ln)
		if ln == "" {
			continue
		}
		parts := strings.Split(ln, "\t")
		if len(parts) < 3 {
			continue
		}
		// binary changes show "-" in numstat
		if parts[0] != "-" {
			var a int
			_, _ = fmt.Sscanf(parts[0], "%d", &a)
			ns.add += a
		}
		if parts[1] != "-" {
			var d int
			_, _ = fmt.Sscanf(parts[1], "%d", &d)
			ns.del += d
		}
		ns.n++
	}
	return ns
}

func gitDiffUnifiedLimited(baseRef string) ([]byte, bool, error) {
	if !gitBaseExists(baseRef) {
		return []byte("Diff: n/a (base-ref not found)\n"), false, nil
	}
	b, err := gitCmd("diff", gitNoExtDiff, baseRef+gitDiffHeadSuffix)
	if err != nil {
		return nil, false, err
	}
	if len(b) <= maxAIPackDiffBytes {
		return b, false, nil
	}
	// truncate
	trunc := b[:maxAIPackDiffBytes]
	trunc = append(trunc, []byte("\n\n---\nTRUNCATED: diff exceeded max size\n")...)
	return trunc, true, nil
}
