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
	cockpit dogfood

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
	outDir, err := cockpit.Dogfood()
	if err != nil {
		fmt.Fprintln(os.Stderr, err.Error())
		os.Exit(1)
	}
	fmt.Printf("Dogfood generated in: %s\n", outDir)
}
