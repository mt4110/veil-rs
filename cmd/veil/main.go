package main

import (
	"fmt"
	"os"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(1)
	}

	cmd := os.Args[1]
	args := os.Args[2:]

	// Default Context
	cwd, _ := os.Getwd()
	ctx := &AppContext{
		Stdout: os.Stdout,
		Stderr: os.Stderr,
		FS:     os.DirFS(cwd),
		Now:    time.Now().UTC(),
	}

	switch cmd {
	case "exceptions":
		handleExceptions(ctx, args)
	default:
		usage()
		os.Exit(1)
	}
}

func usage() {
	fmt.Fprintf(os.Stderr, "Usage: veil <command> [args]\n")
	fmt.Fprintf(os.Stderr, "Commands:\n")
	fmt.Fprintf(os.Stderr, "  exceptions list [--status <active|expiring_soon|expired>] [--format <table|json>]\n")
	fmt.Fprintf(os.Stderr, "  exceptions show <id>\n")
}

func handleExceptions(ctx *AppContext, args []string) {
	if len(args) < 1 {
		usage()
		os.Exit(1)
	}
	subCmd := args[0]
	// Correctly slice args for subcommand flags
	// veil exceptions list --status ...
	// args[0] = list
	// args[1:] = flags
	
	subArgs := args[1:]

	var exitCode int
	switch subCmd {
	case "list":
		exitCode = runExceptionsList(ctx, subArgs)
	case "show":
		if len(subArgs) < 1 {
			fmt.Fprintf(ctx.Stderr, "Error: show requires an ID\n")
			os.Exit(1)
		}
		exitCode = runExceptionsShow(ctx, subArgs[0])
	default:
		usage()
		os.Exit(1)
	}
	
	if exitCode != 0 {
		os.Exit(exitCode)
	}
}
// Removed inline runExceptionsList and runExceptionsShow as they are now in exceptions.go
