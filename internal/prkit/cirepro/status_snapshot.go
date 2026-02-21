package cirepro

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// buildStatusSnapshot extracts S12 rows from docs/ops/STATUS.md.
func buildStatusSnapshot(tsUTC, runID string, gi GitInfo, repoRoot string) string {
	b := &strings.Builder{}
	fmt.Fprintf(b, "timestamp_utc: %s\n", tsUTC)
	fmt.Fprintf(b, "run_id: %s\n", runID)
	fmt.Fprintf(b, "git_sha: %s\n", gi.SHA)
	fmt.Fprintf(b, "git_tree: %s\n", gi.TreeStatus)
	fmt.Fprintf(b, "repo_root: %s\n\n", repoRoot)

	if repoRoot == "" {
		b.WriteString("ERROR: repo root unknown; cannot read docs/ops/STATUS.md\n")
		return b.String()
	}

	p := filepath.Join(repoRoot, "docs", "ops", "STATUS.md")
	f, err := os.Open(p)
	if err != nil {
		fmt.Fprintf(b, "ERROR: cannot open STATUS.md: %v\n", err)
		return b.String()
	}
	defer f.Close()

	sc := bufio.NewScanner(f)
	found := false
	for sc.Scan() {
		line := sc.Text()
		if strings.HasPrefix(line, "| S12-") {
			found = true
			b.WriteString(line + "\n")
		}
	}
	if !found {
		b.WriteString("ERROR: no S12 rows found\n")
	}
	return b.String()
}
