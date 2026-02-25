package main

import (
	"testing"
)

func TestScanEvidenceContent(t *testing.T) {
	tests := []struct {
		name    string
		content string
		wantErr bool
	}{
		{
			name:    "safe content",
			content: "This is a safe PR verify output\nAll tests passed.",
			wantErr: false,
		},
		{
			name:    "safe url with https",
			content: "See https://github.com/foo/bar/../baz for details",
			wantErr: false,
		},
		{
			name:    "safe absolute path string in url",
			content: "Link: https://example.com/Users/admin/info",
			wantErr: false,
		},
		{
			name:    "binary file with NUL byte",
			content: "ELF\x00\x01\x02\x03\x04... some binary data /Users/foo",
			wantErr: false, // Should be skipped
		},
		{
			name:    "forbidden file:// scheme",
			content: "Found at file:///Users/test/workspace/file.txt",
			wantErr: true,
		},
		{
			name:    "forbidden file:/ scheme",
			content: "See file:/home/user/log.txt",
			wantErr: true,
		},
		{
			name:    "forbidden file:\\ scheme",
			content: "Path: file:\\C:\\Logs",
			wantErr: true,
		},
		{
			name:    "forbidden parent traversal at start of line",
			content: "List of files:\n../config.json",
			wantErr: true,
		},
		{
			name:    "forbidden parent traversal with space",
			content: "cat ../secrets.txt",
			wantErr: true,
		},
		{
			name:    "forbidden absolute path /Users/",
			content: "Error in /Users/dev/project/main.go",
			wantErr: true,
		},
		{
			name:    "forbidden absolute path /home/",
			content: "Path log: `/home/runner/work/repo`",
			wantErr: true, // bounded by backtick
		},
		{
			name:    "forbidden Windows drive C:\\",
			content: "Crash dump saved to C:\\Windows\\Temp",
			wantErr: true,
		},
		{
			name:    "forbidden Windows drive D:/",
			content: "Volume: D:/Data",
			wantErr: true,
		},
		{
			name:    "safe word containing /etc/ not at boundary",
			content: "Check out this/etc/thing",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := scanEvidenceContent("test.md", []byte(tt.content))
			if (err != nil) != tt.wantErr {
				t.Errorf("scanEvidenceContent() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
