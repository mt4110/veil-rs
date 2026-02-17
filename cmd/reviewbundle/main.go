package main

import (
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
		// TODO: flags for mode/out-dir
		mode := "wip"
		outDir := ".local/review-bundles"
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
}
