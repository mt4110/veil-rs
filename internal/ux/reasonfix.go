package ux

import (
	"fmt"
	"io"
	"os"
	"sort"
)

// RFv1 Constants
const (
	Prefix = "COCKPIT_RFv1: "
)

// Result represents the high-level outcome of a step
type Result struct {
	Step   string
	Status string // "PASS" or "FAIL"
}

func (r Result) String() string {
	return fmt.Sprintf("%sRESULT=%s STEP=%s", Prefix, r.Status, r.Step)
}

// Reason represents why something failed
type Reason struct {
	Code    string
	Message string
}

func (r Reason) String() string {
	return fmt.Sprintf("%sREASON code=%s msg=%q", Prefix, r.Code, r.Message)
}

// Fix represents how to resolve the failure
type Fix struct {
	Code    string
	Message string
}

func (f Fix) String() string {
	return fmt.Sprintf("%sFIX    code=%s msg=%q", Prefix, f.Code, f.Message)
}

// Docs points to the SSOT
type Docs struct {
	Code      string
	Reference string
}

func (d Docs) String() string {
	return fmt.Sprintf("%sDOCS   code=%s ref=%q", Prefix, d.Code, d.Reference)
}

// Output holds the collection of reasons, fixes, and docs
type Output struct {
	Step    string
	Status  string
	Reasons []Reason
	Fixes   []Fix
	Docs    []Docs
}

// Add adds a failure tuple (Reason, Fix, Doc) to the output
func (o *Output) Add(code, reason, fix, docRef string) {
	o.Reasons = append(o.Reasons, Reason{Code: code, Message: reason})
	o.Fixes = append(o.Fixes, Fix{Code: code, Message: fix})
	o.Docs = append(o.Docs, Docs{Code: code, Reference: docRef})
	o.Status = "FAIL"
}

// PrintTo outputs the RFv1 lines deterministically sorted by Code to the writer
func (o *Output) PrintTo(w io.Writer) {
	// Print Result Line
	fmt.Fprintln(w, Result{Step: o.Step, Status: o.Status})

	// Sort by Code to ensure deterministic output
	sort.Slice(o.Reasons, func(i, j int) bool { return o.Reasons[i].Code < o.Reasons[j].Code })
	sort.Slice(o.Fixes, func(i, j int) bool { return o.Fixes[i].Code < o.Fixes[j].Code })
	sort.Slice(o.Docs, func(i, j int) bool { return o.Docs[i].Code < o.Docs[j].Code })

	for _, r := range o.Reasons {
		fmt.Fprintln(w, r)
	}
	for _, f := range o.Fixes {
		fmt.Fprintln(w, f)
	}
	for _, d := range o.Docs {
		fmt.Fprintln(w, d)
	}

	// Human Readable Block
	fmt.Fprintln(w)
	if o.Status == "FAIL" {
		fmt.Fprintf(w, "✗ cockpit %s failed\n\n", o.Step)
	} else {
		fmt.Fprintf(w, "✓ cockpit %s passed\n\n", o.Step)
		return
	}

	if len(o.Reasons) > 0 {
		fmt.Fprintln(w, "Reason:")
		for _, r := range o.Reasons {
			fmt.Fprintf(w, "- [%s] %s\n", r.Code, r.Message)
		}
		fmt.Fprintln(w)
	}

	if len(o.Fixes) > 0 {
		fmt.Fprintln(w, "Fix:")
		for _, f := range o.Fixes {
			fmt.Fprintf(w, "- %s\n", f.Message)
		}
		fmt.Fprintln(w)
	}

	if len(o.Docs) > 0 {
		fmt.Fprintln(w, "Docs:")
		for _, d := range o.Docs {
			fmt.Fprintf(w, "- %s\n", d.Reference)
		}
	}
}

// Print outputs to stdout
func (o *Output) Print() {
	o.PrintTo(os.Stdout)
}
