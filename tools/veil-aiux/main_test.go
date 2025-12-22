package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// ---- validateVersion ----
func TestValidateVersion(t *testing.T) {
	tests := []struct {
		ver     string
		wantErr bool
	}{
		{"v0.14.0", false},
		{"v1.0.0", false},
		{"v0.0.1", false},
		{"", true},         // empty
		{"0.14.0", true},   // missing v
		{"v0.14", true},    // missing patch
		{"v0.14.0.0", true},// extra dot
		{"v0.14.a", true},  // non-numeric
		{"../v0.14.0", true},// path traversal
		{"v0.1/4.0", true}, // path traversal
	}
	for _, tt := range tests {
		err := validateVersion(tt.ver)
		if (err != nil) != tt.wantErr {
			t.Errorf("validateVersion(%q) error = %v, wantErr %v", tt.ver, err, tt.wantErr)
		}
	}
}

// ---- applyTemplate ----
func TestApplyTemplate(t *testing.T) {
	ver := "v0.14.0"
	tests := []struct {
		name string
		in   string
		want string
	}{
		{
			name: "Normal replacement",
			in:   "Version: {{VERSION}}, Ver: {{VER}}",
			want: "Version: v0.14.0, Ver: v0.14.0",
		},
		{
			name: "Alternative placeholders",
			in:   "Shell: ${VERSION}, Tag: <VER>",
			want: "Shell: v0.14.0, Tag: v0.14.0",
		},
		{
			name: "Safety check (do not replace vX.Y.Z example)",
			in:   "Example: vX.Y.Z",
			want: "Example: vX.Y.Z", // Must NOT change
		},
	}
	for _, tt := range tests {
		got := string(applyTemplate([]byte(tt.in), ver))
		if got != tt.want {
			t.Errorf("%s: applyTemplate() = %q, want %q", tt.name, got, tt.want)
		}
	}
}

// ---- checkMarkdownTemplate ----
func TestCheckMarkdownTemplate(t *testing.T) {
	tests := []struct {
		name    string
		content string
		wantErr bool
	}{
		{
			name:    "Valid template",
			content: "# Title\n\nBody\n```\ncode\n```",
			wantErr: false,
		},
		{
			name:    "Missing H1",
			content: "No header\n",
			wantErr: true,
		},
		{
			name:    "Multiple H1",
			content: "# Title 1\n# Title 2",
			wantErr: true, // exactly 1 allowed
		},
		{
			name:    "Unbalanced Code Fences",
			content: "# Title\n```\nOpen only",
			wantErr: true,
		},
		{
			name:    "Empty",
			content: "",
			wantErr: true,
		},
	}
	for _, tt := range tests {
		err := checkMarkdownTemplate("test.md", []byte(tt.content))
		if (err != nil) != tt.wantErr {
			t.Errorf("%s: checkMarkdownTemplate() error = %v, wantErr %v", tt.name, err, tt.wantErr)
		}
	}
}

// ---- validateDistExactly4 (Integration) ----
func TestValidateDistExactly4(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "veil-test-dist")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	ver := "v0.14.0"

	// Helper to create file
	create := func(name string) {
		_ = os.WriteFile(filepath.Join(tmpDir, name), []byte("content"), 0644)
	}

	// 1. Correct state
	create("PUBLISH_" + ver + ".md")
	create("RELEASE_BODY_" + ver + ".md")
	create("X_" + ver + ".md")
	create("AI_PACK_" + ver + ".txt")

	if err := validateDistExactly4(tmpDir, ver); err != nil {
		t.Errorf("Perfect dist should pass, got error: %v", err)
	}

	// 2. Extra file
	create("EXTRA.txt")
	if err := validateDistExactly4(tmpDir, ver); err == nil {
		t.Errorf("Extra file should fail, got nil")
	}
	_ = os.Remove(filepath.Join(tmpDir, "EXTRA.txt"))

	// 3. Missing file
	_ = os.Remove(filepath.Join(tmpDir, "X_"+ver+".md"))
	if err := validateDistExactly4(tmpDir, ver); err == nil {
		t.Errorf("Missing file should fail, got nil")
	}
	create("X_" + ver + ".md") // restore

	// 4. Forbidden extension (Leak guard)
	create("AI_PACK_" + ver + ".md")
	err = validateDistExactly4(tmpDir, ver)
	if err == nil {
		t.Errorf("AI_PACK.md should fail, got nil")
	} else if !strings.Contains(err.Error(), "forbidden file") {
		t.Errorf("Expected forbidden file error, got: %v", err)
	}
}
