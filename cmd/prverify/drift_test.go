package main

import (
	"strings"
	"testing"
	"testing/fstest"
)

func TestValidateDrift(t *testing.T) {
	validCI := `jobs:
  test:
    steps:
      - run: ops/ci/install_sqlx_cli.sh
      - run: echo ".local/ci/sqlx_cli_install.log"
      - run: echo ".local/ci/sqlx_prepare_check.txt"
      - uses: actions/upload-artifact
        with:
          path: .local/ci/
      - run: : > .local/ci/.keep
`
	validDoc := "SQLX_OFFLINE sqlx_cli_install.log ops/ci/"
	validSOT := "Evidence:\n- sqlx_cli_install.log\n- SQLX_OFFLINE"

	tests := []struct {
		name    string
		fs      fstest.MapFS
		wantErr string
	}{
		// Standard Checks
		{
			name: "Clean / No Drift",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":             {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":              {Data: []byte(validDoc)},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte(validSOT)},
				"ops/exceptions.toml":                  {Data: []byte("")},
			},
			wantErr: "",
		},

		// PR-37 Logic: Max PR Fallback
		{
			name: "SOT Max PR Selection (History exists)",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/PR-33-old.md":     {Data: []byte("old")},
				"docs/pr/PR-34-mid.md":     {Data: []byte("mid")},
				"docs/pr/PR-35-new.md":     {Data: []byte(validSOT)},
				"ops/exceptions.toml":      {Data: []byte("")},
			},
			wantErr: "",
		},
		{
			name: "SOT Ambiguous (Same PR Number)",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/PR-35-a.md":       {Data: []byte(validSOT)},
				"docs/pr/PR-35-b.md":       {Data: []byte(validSOT)},
				"ops/exceptions.toml":      {Data: []byte("")},
			},
			wantErr: "sot_ambiguous",
		},
		{
			name: "SOT Missing (No PR files)",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/README.md":        {Data: []byte("ignore")},
				"docs/pr/PR-TBD-ignore.md": {Data: []byte("ignore")},
				"ops/exceptions.toml":      {Data: []byte("")},
			},
			wantErr: "sot_missing",
		},
		{
			name: "SOT Ignore Invalid Filenames",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":    {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":     {Data: []byte(validDoc)},
				"docs/pr/PR-35-valid.md":      {Data: []byte(validSOT)},
				"docs/pr/PR-35-invalid":       {Data: []byte("no extension")},
				"docs/pr/PR-nobum-invalid.md": {Data: []byte("no num")},
				"ops/exceptions.toml":         {Data: []byte("")},
			},
			wantErr: "",
		},
		{
			name: "Drift Ignored via .driftignore",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/README.md":        {Data: []byte("ignore")},
				".driftignore":             {Data: []byte("# Ignore SOT missing\nsot_missing")},
				"ops/exceptions.toml":      {Data: []byte("")},
			},
			wantErr: "",
		},
		{
			name: "Drift Not Ignored (Mismatch .driftignore)",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/README.md":        {Data: []byte("ignore")},
				".driftignore":             {Data: []byte("other_error")},
				"ops/exceptions.toml":      {Data: []byte("")},
			},
			wantErr: "sot_missing",
		},
		{
			name: "Registry Missing",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":             {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":              {Data: []byte(validDoc)},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte(validSOT)},
			},
			wantErr: "ops/exceptions.toml is missing",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateDrift(tt.fs, 0)
			if tt.wantErr == "" {
				if err != nil {
					t.Errorf("validateDrift() unexpected error: %v", err)
				}
			} else {
				if err == nil {
					t.Errorf("validateDrift() expected error containing %q, got nil", tt.wantErr)
				} else if !strings.Contains(err.Error(), tt.wantErr) {
					t.Errorf("validateDrift() error = %v, want substring %q", err, tt.wantErr)
				}
			}
		})
	}
}

// Unit test for findSOT specific logic (whitebox) to verify Wanted PR logic
func TestFindSOT_Logic(t *testing.T) {
	fsys := fstest.MapFS{
		"docs/pr/PR-33-old.md": {Data: []byte("")},
		"docs/pr/PR-34-mid.md": {Data: []byte("")},
		"docs/pr/PR-35-new.md": {Data: []byte("")},
	}

	// Case 1: Wanted PR exists logic
	// Note: We need to expose findSOT or copy logic if private.
	// Since findSOT is private in main package, we test via internal test in same package.
	// This file is package main, so we can access findSOT.

	// Wanted = 34
	got, err := findSOT(fsys, 34)
	if err != nil {
		t.Fatalf("findSOT(34) failed: %v", err)
	}
	if !strings.Contains(got, "PR-34") {
		t.Errorf("findSOT(34) = %q, want PR-34", got)
	}

	// Wanted = 99 (Missing)
	_, err = findSOT(fsys, 99)
	if err == nil {
		t.Error("findSOT(99) expected error, got nil")
	}

	// Wanted = 0 (Max)
	got, err = findSOT(fsys, 0)
	if err != nil {
		t.Fatalf("findSOT(0) failed: %v", err)
	}
	if !strings.Contains(got, "PR-35") {
		t.Errorf("findSOT(0) = %q, want PR-35 (Max)", got)
	}
}

func TestParseException(t *testing.T) {
	today := "20250101"

	tests := []struct {
		name      string
		substring string
		meta      string
		want      bool // true = ignored, false = failed (expired)
	}{
		{
			name:      "Valid Future Expiry",
			substring: "foo",
			meta:      "reason | until_20250201",
			want:      true,
		},
		{
			name:      "Expired",
			substring: "foo",
			meta:      "reason | until_20241231",
			want:      false,
		},
		{
			name:      "Missing Expiry (Legacy/Invalid but ignored)",
			substring: "foo",
			meta:      "reason only",
			want:      true, // Currently warns but returns true
		},
		{
			name:      "Malformed Expiry Length",
			substring: "foo",
			meta:      "reason | until_2025",
			want:      true, // Warns but returns true
		},
		{
			name:      "Malformed Expiry Non-Digit",
			substring: "foo",
			meta:      "reason | until_2025xxxx",
			want:      true, // Warns but returns true
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := parseException(tt.substring, tt.meta, today); got != tt.want {
				t.Errorf("parseException() = %v, want %v", got, tt.want)
			}
		})
	}
}
