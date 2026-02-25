package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
)

func main() {
	os.Exit(run(os.Args, os.Stdout, os.Stderr))
}

func run(argv []string, stdout, stderr io.Writer) int {
	if len(argv) < 2 {
		usage(stderr)
		fmt.Fprintln(stdout, "ERROR: missing_command")
		fmt.Fprintln(stdout, "OK: phase=end stop=1")
		return 0 // stopless: always exit 0
	}

	cmd := argv[1]
	switch cmd {
	case "verify":
		if len(argv) < 3 {
			fmt.Fprintln(stderr, "error: verify requires bundle path")
			usage(stderr)
			fmt.Fprintln(stdout, "ERROR: verify_args_missing")
			fmt.Fprintln(stdout, "OK: phase=end stop=1")
			return 0 // stopless: always exit 0
		}
		bundlePath := argv[2]
		report, err := VerifyBundlePath(bundlePath, DefaultVerifyOptions)
		if err != nil {
			fmt.Fprintf(stdout, "ERROR: verify_failed path=%s detail=%s\n", bundlePath, err.Error())
			fmt.Fprintln(stderr, err.Error())
			fmt.Fprintln(stdout, "OK: phase=end stop=1")
			return 0 // stopless: always exit 0
		}
		fmt.Fprintf(stdout, "OK: contract=%s mode=%s head=%s epoch=%d\n",
			report.Contract.ContractVersion, report.Contract.Mode,
			report.Contract.HeadSHA[:12], report.Contract.EpochSec)
		fmt.Fprintf(stdout, "PASS: bundle verified path=%s\n", bundlePath)
		fmt.Fprintln(stdout, "OK: phase=end stop=0")
		return 0

	case "create":
		fs := flag.NewFlagSet("create", flag.ContinueOnError)
		fs.SetOutput(stderr)
		modeFlag := fs.String("mode", "", "Bundle mode (strict|wip)")
		outDirFlag := fs.String("out-dir", "", "Output directory")

		heavyFlag := fs.String("heavy", "auto", "Heavy verification mode (auto|never|force)")
		autoCommitFlag := fs.Bool("autocommit", false, "Automatically commit if dirty")
		messageFlag := fs.String("message", "", "Commit message (required if autocommit is true)")
		evidenceReportFlag := fs.String("evidence-report", "", "Explicit path to evidence report (optional)")

		if err := fs.Parse(argv[2:]); err != nil {
			// flag.ContinueOnError means parsing error returns error but doesn't exit.
			// However, NewFlagSet implicitly prints usage on error if we don't suppress it?
			// But we want to handle return code.
			return 1
		}

		mode := *modeFlag
		if mode == "" {
			mode = os.Getenv("MODE")
		}
		if mode == "" {
			mode = "wip"
		}

		outDir := *outDirFlag
		if outDir == "" {
			outDir = os.Getenv("OUT_DIR")
		}
		if outDir == "" {
			outDir = ".local/review-bundles"
		}

		// C2: flags/env for create mode+outdir
		if err := CreateBundleUI(mode, outDir, "", *heavyFlag, *autoCommitFlag, *messageFlag, *evidenceReportFlag, stdout, stderr); err != nil {
			if strings.Contains(err.Error(), "create_generated_invalid_bundle") {
				fmt.Fprintf(stdout, "ERROR: create_generated_invalid_bundle detail=%s\n", err.Error())
				fmt.Fprintln(stdout, "ERROR: create_generated_invalid_bundle stop=1")
			} else {
				fmt.Fprintf(stdout, "ERROR: create_failed detail=%s\n", err.Error())
			}
			fmt.Fprintln(stdout, "OK: phase=end stop=1")
			return 0 // stopless: always exit 0
		}
		fmt.Fprintln(stdout, "OK: create stop=0")
		fmt.Fprintln(stdout, "OK: phase=end stop=0")
		return 0

	default:
		usage(stderr)
		fmt.Fprintf(stdout, "ERROR: unknown_command cmd=%s\n", cmd)
		fmt.Fprintln(stdout, "OK: phase=end stop=1")
		return 0 // stopless: always exit 0
	}
}

func usage(w io.Writer) {
	fmt.Fprintln(w, "usage: reviewbundle <command> [args]")
	fmt.Fprintln(w, "commands:")
	fmt.Fprintln(w, "  verify <path>  Verify a review bundle")
	fmt.Fprintln(w, "  create         Create a new review bundle")
	fmt.Fprintln(w, "    --mode     strict|wip (env: MODE, default: wip)")
	fmt.Fprintln(w, "    --out-dir  Path (env: OUT_DIR, default: .local/review-bundles)")
	fmt.Fprintln(w, "    --heavy    auto|never|force (default: auto)")
	fmt.Fprintln(w, "    --autocommit  (bool) Automatically commit if dirty")
	fmt.Fprintln(w, "    --message     Commit message (required if autocommit is true)")
	fmt.Fprintln(w, "    --evidence-report Path (Optional explicit evidence)")
}
