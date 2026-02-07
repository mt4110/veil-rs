package main

import (
	"strings"
	"testing"
	"testing/fstest"
)

func TestValidateDrift(t *testing.T) {
	// Common valid file contents
	validCI := `
jobs:
  test:
    steps:
      - run: ops/ci/install_sqlx_cli.sh
      - run: echo ".local/ci/sqlx_cli_install.log"
      - run: echo ".local/ci/sqlx_prepare_check.txt"
      - uses: actions/upload-artifact
        with:
          path: .local/ci/
      - run: touch .local/ci/.keep
`
	validDoc := `
This doc mentions SQLX_OFFLINE and sqlx_cli_install.log and even ops/ci/ exception.
`
	validSOT := `
# PR-35 v0.22.0 robust-sqlx
Evidence:
- sqlx_cli_install.log
- SQLX_OFFLINE
`

	tests := []struct {
		name    string
		fs      fstest.MapFS
		wantErr string // substring of expected error, or empty for success
	}{
		{
			name: "Clean / No Drift",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":          {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":           {Data: []byte(validDoc)},
				"docs/ci/prverify.md":               {Data: []byte("extra doc")},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte(validSOT)},
			},
			wantErr: "",
		},
		{
			name: "CI Drift: Missing install script",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":          {Data: []byte("invalid ci")},
				"docs/guardrails/sqlx.md":           {Data: []byte(validDoc)},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte(validSOT)},
			},
			wantErr: "CI Drift: CI must use ops/ci/install_sqlx_cli.sh",
		},
		{
			name: "Docs Drift: Missing keywords",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":          {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":           {Data: []byte("empty")},
				"docs/ci/prverify.md":               {Data: []byte("empty")},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte(validSOT)},
			},
			wantErr: "Docs Drift: Term 'SQLX_OFFLINE' not found",
		},
		{
			name: "SOT Drift: Invalid content",
			fs: fstest.MapFS{
				".github/workflows/ci.yml":          {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":           {Data: []byte(validDoc)},
				"docs/pr/PR-35-v0.22.0-robust-sqlx.md": {Data: []byte("bad content")},
			},
			wantErr: "SOT Drift",
		},
		{
			name: "SOT Missing",
			fs: fstest.MapFS{
				".github/workflows/ci.yml": {Data: []byte(validCI)},
				"docs/guardrails/sqlx.md":  {Data: []byte(validDoc)},
				"docs/pr/readme.md":        {Data: []byte("irrelevant")},
			},
			wantErr: "SOT Drift",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateDrift(tt.fs)
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
