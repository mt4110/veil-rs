package main

import (
	"flag"
	"fmt"
	"os"

	"veil-rs/internal/prkit"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(2)
	}
}

func run() error {
	var dryRun bool
	var format string

	flag.BoolVar(&dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	flag.StringVar(&format, "format", "portable-json", "Output format (default: portable-json)")
	flag.Parse()

	if !dryRun {
		return fmt.Errorf("v1 requires --dry-run")
	}

	if format != "portable-json" {
		return fmt.Errorf("unsupported format: %s", format)
	}

	return prkit.RunDryRun()
}
