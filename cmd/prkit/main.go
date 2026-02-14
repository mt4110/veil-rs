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

	// Validate flag exclusivity
	if sotNew {
		if dryRun || runMode || reviewBundle || outPath != "" {
			fmt.Fprintln(os.Stderr, "Error: --sot-new cannot be combined with --dry-run, --run, --review-bundle, or --out")
			return 2
		}

		// SOT scaffolding execution
		if err := prkit.ScaffoldSOT(epic, slug, release, apply); err != nil {
			fmt.Fprintf(os.Stderr, "failed to scaffold SOT: %v\n", err)
			return 1
		}
		return 0
	}

	// Normal mode (dry-run or run)
	if dryRun && runMode {
		fmt.Fprintln(os.Stderr, "Error: cannot specify both --dry-run and --run")
		return 2
	}
	if !dryRun && !runMode {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("must specify either --dry-run, --run, or --sot-new")); err != nil {
			fmt.Fprintf(os.Stderr, "Error generating failure evidence: %v\n", err)
		}
		return 2
	}

	// Run mode specific flags
	if dryRun {
		if reviewBundle {
			fmt.Fprintln(os.Stderr, "Error: --review-bundle is only supported in --run mode")
			return 2
		}
		if outPath != "" {
			fmt.Println("Warning: --out is ignored in --dry-run mode")
		}
	} else {
		// Run mode
		if outPath == "" {
			if err := prkit.GenerateFailureEvidence(fmt.Errorf("--out is required in --run mode")); err != nil {
				fmt.Fprintf(os.Stderr, "Error generating failure evidence: %v\n", err)
			}
			return 2
		}
	}

	if format != "portable-json" {
		if err := prkit.GenerateFailureEvidence(fmt.Errorf("unsupported format: %s", format)); err != nil {
			fmt.Fprintf(os.Stderr, "Error generating failure evidence: %v\n", err)
		}
		return 2
	}

	// Validation handled above

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
