package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"

	"veil-rs/internal/prkit"
)

const program = "prkit"

func main() {
	os.Exit(Run(os.Args[1:], os.Stdout, os.Stderr, nil))
}

func Run(argv []string, stdout, stderr io.Writer, runner prkit.ExecRunner) int {
	prkit.ResetTrace()

	// Initialize ExecRunner
	// If runner is provided (testing), inject it immediately to ensure FindRepoRoot uses it.
	if runner != nil {
		prkit.Init("", runner)
	}

	// Try to find repo root (using whatever Runner is currently set)
	if root, err := prkit.FindRepoRoot(); err == nil {
		// Re-init with found root.
		prkit.Init(root, runner)
	} else {
		// If failed to find root, we stick with current runner config.
	}

	fs := flag.NewFlagSet(program, flag.ContinueOnError)

	// flagパッケージが勝手にstderrに吐くと、stdoutのportable-jsonと混線して地獄になるので握りつぶす。
	// 使い方/エラー表示は自前で固定出力する。
	fs.SetOutput(io.Discard)

	// Normal modes
	var dryRun bool
	var runMode bool
	var reviewBundle bool
	var outPath string
	var format string

	// S10-05: SOT scaffolding
	var sotNew bool
	var epic string
	var slug string
	var release string
	var apply bool

	// Help (explicit, deterministic)
	var help bool

	fs.BoolVar(&help, "help", false, "Show help")
	fs.BoolVar(&help, "h", false, "Show help (shorthand)")

	fs.BoolVar(&dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	fs.BoolVar(&runMode, "run", false, "Enable execution mode")
	fs.BoolVar(&reviewBundle, "review-bundle", false, "Generate review bundle (run mode only)")
	fs.StringVar(&outPath, "out", "", "Output path for evidence (run mode only)")
	fs.StringVar(&format, "format", "portable-json", "Output format (default: portable-json)")

	fs.BoolVar(&sotNew, "sot-new", false, "Generate SOT scaffolding")
	fs.StringVar(&epic, "epic", "", "Epic name (for SOT)")
	fs.StringVar(&slug, "slug", "", "PR slug (for SOT)")
	fs.StringVar(&release, "release", "", "Release version (for SOT, optional/autodetect)")
	fs.BoolVar(&apply, "apply", false, "Apply SOT scaffolding (write file)")

	usage := func() {
		fmt.Fprintf(stderr, "Usage of %s:\n", program)
		// PrintDefaultsはFlagSetのOutputに書くので、一時的にstderrへ向ける
		fs.SetOutput(stderr)
		fs.PrintDefaults()
		fs.SetOutput(io.Discard)
	}

	fail := func(err error) int {
		// 人間向けはstderr
		if err != nil {
			fmt.Fprintln(stderr, err.Error())
		}
		// 監査向けはstdout（portable-json）
		if genErr := prkit.GenerateFailureEvidence(err, stdout); genErr != nil {
			fmt.Fprintf(stderr, "failed to generate failure evidence: %v\n", genErr)
		}
		return 2
	}

	// Parse
	if err := fs.Parse(argv); err != nil {
		// ContinueOnError + io.Discard なので、自前でusageを出す
		usage()
		return fail(err)
	}

	// Helpは「即return」。ここが今回の止血ポイント。
	if help {
		usage()
		return 0
	}

	// Validate flag exclusivity (SOT)
	if sotNew {
		if dryRun || runMode || reviewBundle || outPath != "" {
			return fail(errors.New("Error: --sot-new cannot be combined with --dry-run, --run, --review-bundle, or --out"))
		}

		if epic == "" || slug == "" {
			return fail(errors.New("Error: --sot-new requires --epic and --slug"))
		}

		if err := prkit.ScaffoldSOT(epic, slug, release, apply); err != nil {
			// scaffold失敗は「実行エラー」なので fail() に委譲（JSON + stderr）
			return fail(fmt.Errorf("failed to scaffold SOT: %w", err))
		}
		return 0
	}

	// Normal mode (dry-run or run)
	if dryRun && runMode {
		return fail(errors.New("Error: cannot specify both --dry-run and --run"))
	}
	if !dryRun && !runMode {
		return fail(fmt.Errorf("must specify either --dry-run, --run, or --sot-new"))
	}

	// Run mode specific flags
	if dryRun {
		if reviewBundle {
			return fail(errors.New("Error: --review-bundle is only supported in --run mode"))
		}
		if outPath != "" {
			fmt.Fprintln(stderr, "Warning: --out is ignored in --dry-run mode")
		}
	} else {
		// Run mode
		if outPath == "" {
			return fail(fmt.Errorf("--out is required in --run mode"))
		}
	}

	if format != "portable-json" {
		return fail(fmt.Errorf("unsupported format: %s", format))
	}

	var exitCode int
	var err error

	if dryRun {
		exitCode, err = prkit.RunDryRun(stdout)
	} else {
		exitCode, err = prkit.RunExecuteMode(outPath, reviewBundle, stdout)
	}

	if err != nil {
		return fail(err)
	}
	return exitCode
}
