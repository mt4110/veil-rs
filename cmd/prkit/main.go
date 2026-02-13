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
	var runMode bool
	var reviewBundle bool
	var outPath string
	var format string

	flag.BoolVar(&dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	flag.BoolVar(&runMode, "run", false, "Enable execution mode")
	flag.BoolVar(&reviewBundle, "review-bundle", false, "Generate review bundle (run mode only)")
	flag.StringVar(&outPath, "out", "", "Output path for evidence (run mode only)")
	flag.StringVar(&format, "format", "portable-json", "Output format (default: portable-json)")

	// S10-05: SOT scaffolding
	var sotNew bool
	var epic string
	var slug string
	var release string
	var apply bool
	flag.BoolVar(&sotNew, "sot-new", false, "Generate SOT scaffolding")
	flag.StringVar(&epic, "epic", "", "Epic name (for SOT)")
	flag.StringVar(&slug, "slug", "", "PR slug (for SOT)")
	flag.StringVar(&release, "release", "", "Release version (for SOT, optional/autodetect)")
	flag.BoolVar(&apply, "apply", false, "Apply SOT scaffolding (write file)")

	flag.Parse()

	const failGenErr = "failed to generate failure evidence: %v\n"

	if !dryRun {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("v1 requires --dry-run")); err != nil {
			fmt.Fprintf(os.Stderr, "Error generating failure evidence: %v\n", err)
		}
		return 2
	}

	if format != "portable-json" {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("unsupported format: %s", format)); err != nil {
			fmt.Fprintf(os.Stderr, "Error generating failure evidence: %v\n", err)
		}
		return 2
	}

	if sotNew {
		if err := prkit.ScaffoldSOT(epic, slug, release, apply); err != nil {
			fmt.Fprintf(os.Stderr, "failed to scaffold SOT: %v\n", err)
			return 1
		}
		return 0
	}

	if dryRun && runMode {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("cannot specify both --dry-run and --run")); err != nil {
			fmt.Fprintf(os.Stderr, failGenErr, err)
		}
		return 2
	}

	if !dryRun && !runMode {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("must specify --dry-run or --run")); err != nil {
			fmt.Fprintf(os.Stderr, failGenErr, err)
		}
		return 2
	}

	var exitCode int
	var err error

	if dryRun {
		// Dry-run mode ignores --out
		exitCode, err = prkit.RunDryRun()
	} else {
		// Run mode
		exitCode, err = prkit.RunExecuteMode(outPath, reviewBundle)
	}

	if err != nil {
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}
	return exitCode
}
