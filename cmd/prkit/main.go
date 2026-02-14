package main

import (
	"flag"
	"fmt"
	"os"

	"veil-rs/internal/prkit"
)

func main() {
	os.Exit(run(os.Args[1:]))
}

func run(argv []string) int {
	// Stable name in usage / errors (avoid go-run cache paths).
	fs := flag.NewFlagSet("prkit", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	// Stable, portable usage header.
	fs.Usage = func() {
		fmt.Fprintln(os.Stderr, "Usage of prkit:")
		fs.PrintDefaults()
	}

	// Flags (normal mode)
	var dryRun bool
	var runMode bool
	var reviewBundle bool
	var outPath string
	var format string

	fs.BoolVar(&dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	fs.BoolVar(&runMode, "run", false, "Enable execution mode")
	fs.BoolVar(&reviewBundle, "review-bundle", false, "Generate review bundle (run mode only)")
	fs.StringVar(&outPath, "out", "", "Output path for evidence (run mode only)")
	fs.StringVar(&format, "format", "portable-json", "Output format (default: portable-json)")

	// SOT scaffolding mode
	var sotNew bool
	var epic string
	var slug string
	var release string
	var apply bool

	fs.BoolVar(&sotNew, "sot-new", false, "Generate SOT scaffolding")
	fs.StringVar(&epic, "epic", "", "Epic name (for SOT)")
	fs.StringVar(&slug, "slug", "", "PR slug (for SOT)")
	fs.StringVar(&release, "release", "", "Release version (for SOT, optional/autodetect)")
	fs.BoolVar(&apply, "apply", false, "Apply SOT scaffolding (write file)")

	// Deterministic help: exit=0, no evidence JSON.
	for _, a := range argv {
		if a == "--help" || a == "-h" || a == "-help" {
			fs.Usage()
			return 0
		}
	}

	if err := fs.Parse(argv); err != nil {
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}

	// SOT mode: isolated from run/dry-run/evidence flags.
	if sotNew {
		if dryRun || runMode || reviewBundle || outPath != "" {
			fmt.Fprintln(os.Stderr, "Error: --sot-new cannot be combined with --dry-run, --run, --review-bundle, or --out")
			return 2
		}
		if err := prkit.ScaffoldSOT(epic, slug, release, apply); err != nil {
			fmt.Fprintf(os.Stderr, "failed to scaffold SOT: %v\n", err)
			return 1
		}
		return 0
	}

	// Normal mode (dry-run or run)
	if dryRun && runMode {
		err := fmt.Errorf("cannot specify both --dry-run and --run")
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}
	if !dryRun && !runMode {
		err := fmt.Errorf("must specify either --dry-run, --run, or --sot-new")
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}

	// Run mode specific flags
	if dryRun {
		if reviewBundle {
			err := fmt.Errorf("--review-bundle is only supported in --run mode")
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			_ = prkit.GenerateFailureEvidence(err)
			return 2
		}
		// --out is ignored in dry-run; keep behavior operator-friendly but deterministic.
		if outPath != "" {
			fmt.Fprintln(os.Stderr, "Warning: --out is ignored in --dry-run mode")
		}
	} else {
		// Run mode
		if outPath == "" {
			err := fmt.Errorf("--out is required in --run mode")
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			_ = prkit.GenerateFailureEvidence(err)
			return 2
		}
	}

	if format != "portable-json" {
		err := fmt.Errorf("unsupported format: %s", format)
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}

	var exitCode int
	var err error

	if dryRun {
		exitCode, err = prkit.RunDryRun()
	} else {
		exitCode, err = prkit.RunExecuteMode(outPath, reviewBundle)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		_ = prkit.GenerateFailureEvidence(err)
		return 2
	}
	return exitCode
}
