package main

import (
	"flag"
	"fmt"
	"os"

	"veil-rs/internal/prkit"
)

func main() {
	os.Exit(run())
}

func run() int {
	var dryRun bool
	var format string

	flag.BoolVar(&dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	flag.StringVar(&format, "format", "portable-json", "Output format (default: portable-json)")
	flag.Parse()

	if !dryRun {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("v1 requires --dry-run")); err != nil {
			fmt.Fprintf(os.Stderr, "failed to generate failure evidence: %v\n", err)
		}
		return 2
	}

	if format != "portable-json" {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("unsupported format: %s", format)); err != nil {
			fmt.Fprintf(os.Stderr, "failed to generate failure evidence: %v\n", err)
		}
		return 2
	}

	exitCode, err := prkit.RunDryRun()
	if err != nil {
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}
	return exitCode
}
