package main

import (
	"flag"
	"fmt"
	"io"
	"os"
)

func main() {
	os.Exit(run(os.Args, os.Stdout, os.Stderr))
}

func run(argv []string, stdout, stderr io.Writer) int {
	if len(argv) < 2 {
		usage(stderr)
		return 1
	}

	cmd := argv[1]
	switch cmd {
	case "verify":
		if len(argv) < 3 {
			fmt.Fprintln(stderr, "error: verify requires bundle path")
			usage(stderr)
			return 1
		}
		path := argv[2]
		report, err := VerifyBundlePath(path)
		if err != nil {
			fmt.Fprintln(stderr, err.Error())
			return 1
		}
		fmt.Fprintf(stdout, "PASS: %s (mode=%s, epoch=%d)\n", path, report.Contract.Mode, report.Contract.EpochSec)
		return 0

	case "create":
		fs := flag.NewFlagSet("create", flag.ContinueOnError)
		fs.SetOutput(stderr)
		modeFlag := fs.String("mode", "", "Bundle mode (strict|wip)")
		outDirFlag := fs.String("out-dir", "", "Output directory")

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
		if err := CreateBundleUI(mode, outDir, "", stdout, stderr); err != nil {
			fmt.Fprintln(stderr, err.Error())
			return 1
		}
		return 0

	default:
		usage(stderr)
		return 1
	}
}

func usage(w io.Writer) {
	fmt.Fprintln(w, "usage: reviewbundle <command> [args]")
	fmt.Fprintln(w, "commands:")
	fmt.Fprintln(w, "  verify <path>  Verify a review bundle")
	fmt.Fprintln(w, "  create         Create a new review bundle")
	fmt.Fprintln(w, "    --mode     strict|wip (env: MODE, default: wip)")
	fmt.Fprintln(w, "    --out-dir  Path (env: OUT_DIR, default: .local/review-bundles)")
}
