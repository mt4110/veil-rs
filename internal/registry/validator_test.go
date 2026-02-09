package registry

import (
	"strings"
	"testing"
	"time"
)

func TestValidate_MissingFields(t *testing.T) {
	today := time.Date(2026, 2, 8, 0, 0, 0, 0, time.UTC)
	reg := &Registry{
		Exceptions: []Exception{{ID: ""}}, // Missing ID -> key "idx 0"
	}
	errs := Validate(reg, today)
	if len(errs) == 0 {
		t.Error("Expected errors for missing fields, got none")
	}
	foundRule := false
	for _, e := range errs {
		if strings.Contains(e.Error(), "missing rule") {
			foundRule = true
		}
	}
	if !foundRule {
		t.Errorf("Expected 'missing rule' error, got: %v", errs)
	}
}

func TestValidate_Expiry(t *testing.T) {
	today := time.Date(2026, 2, 8, 0, 0, 0, 0, time.UTC)

	tests := []struct {
		name    string
		expires string
		wantErr bool
	}{
		{"Future", "2026-03-01", false},
		{"Past", "2026-02-07", true},
		{"Today", "2026-02-08", false}, // Inclusive? Logic says: if now.After(expires) -> Expired. 
		// Now=2026-02-08 00:00:00. Expires=2026-02-08 00:00:00.
		// After is false. So Today is valid.
		{"None", "", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ex := Exception{
				ID:        "TEST",
				Rule:      "r",
				Scope:     "path:s",
				Reason:    "r",
				Owner:     "o",
				CreatedAt: "2026-01-01",
				Audit:     []string{"a"},
				ExpiresAt: tt.expires,
			}
			reg := &Registry{Exceptions: []Exception{ex}}
			errs := Validate(reg, today)
			
			hasErr := false
			for _, e := range errs {
				if strings.Contains(e.Error(), "expired") {
					hasErr = true
				}
			}

			if tt.wantErr && !hasErr {
				t.Errorf("Expected expiry error, got none")
			}
			if !tt.wantErr && hasErr {
				t.Errorf("Unexpected expiry error: %v", errs)
			}
		})
	}
}

func TestValidate_SortStability(t *testing.T) {
	today := time.Date(2026, 2, 8, 0, 0, 0, 0, time.UTC)
	
	// Create registry with mixed IDs and missing IDs
	// We want to ensure Validate returns errors in a deterministic order
	// matching (ID asc, OriginalIndex asc)
	
	exs := []Exception{
		{ID: "", OriginalIndex: 0}, // Missing ID 1
		{ID: "B", OriginalIndex: 1, Rule: "r", Scope: "bad", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}}, // Error: bad scope
		{ID: "A", OriginalIndex: 2, Rule: "r", Scope: "bad", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}}, // Error: bad scope
		{ID: "", OriginalIndex: 3}, // Missing ID 2
	}
	
	reg := &Registry{Exceptions: exs}
	errs := Validate(reg, today)
	
	// Expected Order of ERRORS:
	// 1. ID="" (Orig 0) -> "idx 0: missing id" ...
	// 2. ID="" (Orig 3) -> "idx 3: missing id" ...
	// 3. ID="A" (Orig 2) -> "A: invalid scope ..."
	// 4. ID="B" (Orig 1) -> "B: invalid scope ..."
	
	if len(errs) == 0 {
		t.Fatal("Expected errors")
	}
	
	// Just check the first error of each grouping to verify sort order logic implicitly
	// We scan the errors and extract the "keys"
	
	var keys []string
	seen := make(map[string]bool)
	for _, e := range errs {
		msg := e.Error()
		parts := strings.Split(msg, ":")
		key := parts[0]
		if !seen[key] {
			keys = append(keys, key)
			seen[key] = true
		}
	}
	
	expectedKeys := []string{"idx 0", "idx 3", "A", "B"}
	
	if len(keys) != len(expectedKeys) {
		t.Fatalf("Keys mismatch. Got %v, Want %v", keys, expectedKeys)
	}
	
	for i, k := range keys {
		if k != expectedKeys[i] {
			t.Errorf("Key order mismatch at %d: Got %s, Want %s", i, k, expectedKeys[i])
		}
	}
}
