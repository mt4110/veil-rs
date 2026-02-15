package main

import (
	"io"
	"os"

	"veil-rs/internal/prkit"
)

const program = "prkit"

func main() {
	os.Exit(Run(os.Args[1:], os.Stdout, os.Stderr, nil))
}

func Run(argv []string, stdout, stderr io.Writer, runner prkit.ExecRunner) int {
	return prkit.Run(argv, stdout, stderr, runner)
}
