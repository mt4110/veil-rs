package main

import (
	"fmt"
	"os"
	"path/filepath"

	"veil-rs/internal/cockpit"
)

func usage() {
	fmt.Fprint(os.Stderr, `cockpit - unified repo operations

Usage:
	cockpit status
	cockpit ai-pack [base_ref] [out]
	cockpit gen vX.Y.Z [base_ref]
	cockpit dogfood weekly
	cockpit dogfood analyze [--week-id YYYY-Www]

Exit codes:
	0 success
	1 failure
`)
}

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(1)
	}

	switch os.Args[1] {
	case "status":
		runStatus()
	case "ai-pack":
		runAiPack()
	case "gen":
		runGen()
	case "dogfood":
		runDogfood()
	default:
		usage()
		os.Exit(1)
	}
}

func runStatus() {
	out, err := cockpit.Status()
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}
	fmt.Print(out)
}

func runAiPack() {
	baseRef := ""
	outPath := ""
	if len(os.Args) > 2 {
		baseRef = os.Args[2]
	}
	if len(os.Args) > 3 {
		outPath = os.Args[3]
	}
	finalPath, usedTemp, err := cockpit.GenerateAIPack(baseRef, outPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}
	if usedTemp {
		fmt.Println(finalPath)
	}
}

func runGen() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "Error: version argument is required (e.g. vX.Y.Z)")
		usage()
		os.Exit(1)
	}
	version := os.Args[2]
	baseRef := ""
	if len(os.Args) > 3 {
		baseRef = os.Args[3]
	}
	outDir, err := cockpit.GenerateDrafts(version, baseRef)
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}
	// Print generated files for user info (parity with script)
	fmt.Println("Generated drafts in:", outDir)
	walkErr := filepath.WalkDir(outDir, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			// Warn but continue
			fmt.Fprintf(os.Stderr, "warning: failed to access path %q: %v\n", path, err)
			return nil
		}
		if !d.IsDir() {
			fmt.Println("  " + path)
		}
		return nil
	})
	if walkErr != nil {
		fmt.Fprintln(os.Stderr, "warning: directory walk incomplete:", walkErr)
	}
}

func runDogfood() {
	// Require subcommand
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "Error: missing subcommand. Usage: cockpit dogfood [weekly|analyze]")
		usage()
		os.Exit(2)
	}

	mode := os.Args[2]
	
	// Optional flags
	var weekID string
	// Simple arg parsing for week-id if present
	// expected: cockpit dogfood analyze --week-id 2025-W52
	for i, arg := range os.Args {
		if arg == "--week-id" && i+1 < len(os.Args) {
			weekID = os.Args[i+1]
			break
		}
	}
	if weekID == "" {
		weekID = os.Getenv("WEEK_ID")
	}

	switch mode {
	case "weekly":
		outDir, exitCode, err := cockpit.Dogfood(weekID)
		if err != nil {
			fmt.Fprintln(os.Stderr, err.Error())
			if exitCode == 0 {
				exitCode = 1 // Fallback
			}
			os.Exit(exitCode)
		}
		fmt.Printf("Dogfood generated in: %s\n", outDir)
		os.Exit(0)
	case "analyze":
		if err := cockpit.Analyze(weekID); err != nil {
			fmt.Fprintf(os.Stderr, "Analysis failed: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("Analysis complete.")
		os.Exit(0)
	default:
		fmt.Fprintf(os.Stderr, "Error: unknown dogfood subcommand %q\n", mode)
		os.Exit(2)
	}
}
