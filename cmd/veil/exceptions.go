package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"io/fs"
	"time"

	"veil-rs/internal/registry"
)

type AppContext struct {
	Stdout io.Writer
	Stderr io.Writer
	FS     fs.FS
	Now    time.Time
}

func runExceptionsList(ctx *AppContext, args []string) int {
	fs := flag.NewFlagSet("list", flag.ContinueOnError)
	fs.SetOutput(ctx.Stderr)
	statusFilter := fs.String("status", "", "Filter by status (active, expiring_soon, expired)")
	format := fs.String("format", "table", "Output format (table, json)")
	
	if err := fs.Parse(args); err != nil {
		return 1
	}

	// Strict Status Validation
	if *statusFilter != "" {
		valid := map[string]bool{
			"active":        true,
			"expiring_soon": true,
			"expired":       true,
		}
		if !valid[*statusFilter] {
			fmt.Fprintf(ctx.Stderr, "Invalid status filter: %s\n", *statusFilter)
			return 1
		}
	}

	loader := &registry.Loader{FS: ctx.FS}
	utcToday := time.Date(ctx.Now.Year(), ctx.Now.Month(), ctx.Now.Day(), 0, 0, 0, 0, time.UTC)

	reg, err := loader.Load(utcToday)
	if err != nil {
		fmt.Fprintf(ctx.Stderr, "Error loading registry: %v\n", err)
		return 1
	}

	// Filter
	var filtered []registry.Exception
	for _, ex := range reg.Exceptions {
		if *statusFilter != "" && string(ex.Status) != *statusFilter {
			continue
		}
		filtered = append(filtered, ex)
	}

	if *format == "json" {
		type jsonEntry struct {
			ID      string `json:"id"`
			Status  string `json:"status"`
			Expires string `json:"expires"`
			Owner   string `json:"owner"`
			Reason  string `json:"reason"`
		}
		var output []jsonEntry
		for _, ex := range filtered {
			expires := ex.ExpiresAt
			// SOT: "Perpetual expires ... fixed specification (e.g. empty or null)"
			// If empty string in TOML, it's empty string here.
			// Let's keep it as empty string for JSON if it's empty.
			
			output = append(output, jsonEntry{
				ID:      ex.ID,
				Status:  string(ex.Status),
				Expires: expires,
				Owner:   ex.Owner,
				Reason:  ex.Reason,
			})
		}
		
		// Ensure empty array is [] not null
		if output == nil {
			output = []jsonEntry{}
		}

		enc := json.NewEncoder(ctx.Stdout)
		enc.SetIndent("", "  ")
		if err := enc.Encode(output); err != nil {
			fmt.Fprintf(ctx.Stderr, "Error encoding JSON: %v\n", err)
			return 1
		}
		return 0
	}

	// Table Output
	fmt.Fprintf(ctx.Stdout, "%-18s %-15s %-12s %-10s %s\n", "ID", "STATUS", "EXPIRES", "OWNER", "REASON")
	for _, ex := range filtered {
		reason := ex.Reason
		if len(reason) > 40 {
			reason = reason[:37] + "..."
		}
		expires := ex.ExpiresAt
		if expires == "" {
			expires = "Perpetual" 
		}
		
		fmt.Fprintf(ctx.Stdout, "%-18s %-15s %-12s %-10s %s\n", ex.ID, ex.Status, expires, ex.Owner, reason)
	}
	return 0
}

func runExceptionsShow(ctx *AppContext, id string) int {
	loader := &registry.Loader{FS: ctx.FS}
	utcToday := time.Date(ctx.Now.Year(), ctx.Now.Month(), ctx.Now.Day(), 0, 0, 0, 0, time.UTC)

	reg, err := loader.Load(utcToday)
	if err != nil {
		fmt.Fprintf(ctx.Stderr, "Error loading registry: %v\n", err)
		return 1
	}

	for _, ex := range reg.Exceptions {
		if ex.ID == id {
			fmt.Fprintf(ctx.Stdout, "ID:        %s\n", ex.ID)
			fmt.Fprintf(ctx.Stdout, "Status:    %s\n", ex.Status)
			fmt.Fprintf(ctx.Stdout, "Rule:      %s\n", ex.Rule)
			fmt.Fprintf(ctx.Stdout, "Scope:     %s\n", ex.Scope)
			fmt.Fprintf(ctx.Stdout, "Created:   %s\n", ex.CreatedAt)
			fmt.Fprintf(ctx.Stdout, "Expires:   %s\n", ex.ExpiresAt)
			fmt.Fprintf(ctx.Stdout, "Owner:     %s\n", ex.Owner)
			fmt.Fprintf(ctx.Stdout, "Reason:    %s\n", ex.Reason)
			fmt.Fprintf(ctx.Stdout, "Audit:\n")
			for _, a := range ex.Audit {
				fmt.Fprintf(ctx.Stdout, "  - %s\n", a)
			}
			return 0
		}
	}

	fmt.Fprintf(ctx.Stderr, "Exception not found: %s\n", id)
	return 1
}
