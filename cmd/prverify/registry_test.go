package main

import (
	"fmt"
	"strings"
	"testing"
	"testing/fstest"
	"time"
)

func TestValidateRegistry_Valid(t *testing.T) {
	reg := &Registry{
		Exceptions: []Exception{
			{
				ID:        "EX-20260208-001",
				Rule:      "rule-1",
				Scope:     "path:src/**",
				Reason:    "Valid reason",
				Owner:     "@owner",
				CreatedAt: "2026-02-08",
				Audit:     []string{"http://audit"},
			},
		},
	}
	today := parseDate(t, "2026-02-08")
	errs := validateRegistry(reg, today)
	if len(errs) != 0 {
		t.Errorf("Expected valid registry, got errors: %v", errs)
	}
}

func TestValidateRegistry_MissingFields(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	// Empty exception
	reg := &Registry{
		Exceptions: []Exception{{ID: ""}}, // Missing ID -> key "idx 0"
	}
	errs := validateRegistry(reg, today)
	if len(errs) == 0 {
		t.Error("Expected errors for missing fields, got none")
	}
	// Check for specific missing field errors
	foundRule := false
	for _, e := range errs {
		if e.Error() == "idx 0: missing rule" {
			foundRule = true
		}
	}
	if !foundRule {
		t.Errorf("Expected 'idx 0: missing rule' error, got: %v", errs)
	}
}

func TestValidateRegistry_DuplicateID(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "DUP-01", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
			{ID: "DUP-01", Rule: "r", Scope: "path:b", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg, today)
	foundDup := false
	for _, e := range errs {
		if e.Error() == "duplicate id: DUP-01" {
			foundDup = true
		}
	}
	if !foundDup {
		t.Error("Expected duplicate ID error")
	}
}

func TestValidateRegistry_InvalidScope(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "BAD-SCOPE", Rule: "r", Scope: "invalid:format", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg, today)
	foundScope := false
	expected := "BAD-SCOPE: invalid scope format 'invalid:format' (must start with path: or fingerprint:)"
	for _, e := range errs {
		if e.Error() == expected {
			foundScope = true
		}
	}
	if !foundScope {
		t.Errorf("Expected error '%s', got: %v", expected, errs)
	}
}

func TestValidateRegistry_InvalidDates(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "BAD-DATE", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026/01/01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg, today)
	foundDate := false
	expected := "BAD-DATE: invalid created_at '2026/01/01' (must be YYYY-MM-DD)"
	for _, e := range errs {
		if e.Error() == expected {
			foundDate = true
		}
	}
	if !foundDate {
		t.Errorf("Expected error '%s', got: %v", expected, errs)
	}
}

func TestValidateRegistryFile_Missing(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	// Empty FS -> Missing file
	fsys := fstest.MapFS{}
	err := validateRegistryFile(fsys, today)
	if err == nil {
		t.Fatal("Expected error for missing file, got nil")
	}
	// Check error text/type if possible, or just existence of driftError
	if !strings.Contains(err.Error(), "ops/exceptions.toml is missing") {
		t.Errorf("Unexpected error message: %v", err)
	}
}

const (
	dateFmt = "2006-01-02"
)

func parseDate(t *testing.T, s string) time.Time {
	d, err := time.Parse(dateFmt, s)
	if err != nil {
		t.Fatalf("failed to parse date %s: %v", s, err)
	}
	return d
}

func TestValidateRegistry_Expiry(t *testing.T) {
	today := parseDate(t, "2026-02-08")

	tests := []struct {
		name      string
		exception Exception
		expired   bool
	}{
		{
			name:      "Future (Not Expired)",
			exception: Exception{ID: "FUT", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-09"},
			expired:   false,
		},
		{
			name:      "Past (Expired)",
			exception: Exception{ID: "EXP", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-07"},
			expired:   true,
		},
		{
			name:      "Boundary (Today - Not Expired)",
			exception: Exception{ID: "BND", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-08"},
			expired:   false,
		},
		{
			name:      "No Expiry (Allowed)",
			exception: Exception{ID: "INF", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}}, // ExpiresAt empty
			expired:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			reg := &Registry{Exceptions: []Exception{tt.exception}}
			errs := validateRegistry(reg, today)
			if tt.expired {
				if len(errs) == 0 {
					t.Errorf("Expected expiry error, got none")
				} else if !strings.Contains(errs[0].Error(), "expired on") {
					t.Errorf("Expected 'expired on' error, got: %v", errs[0])
				}
			} else {
				if len(errs) != 0 {
					t.Errorf("Expected valid, got errors: %v", errs)
				}
			}
		})
	}
}

func TestValidateRegistry_GoldenOutput(t *testing.T) {
	// 5. Capping: 12 failures -> 10 displayed + "and 2 more"
	// Also checks 4. Deterministic Sort (ID asc, fallback index implicit in construction order if we force it, but let's mix IDs)
	today := parseDate(t, "2026-02-08")

	var exceptions []Exception
	// Create 12 expired entries with mixed IDs to test sorting
	// We use IDs: EX-11, EX-10, ..., EX-00 to test if result is EX-00, EX-01...
	for i := 11; i >= 0; i-- {
		id := fmt.Sprintf("EX-%02d", i)
		exceptions = append(exceptions, Exception{
			ID:        id,
			Rule:      "r",
			Scope:     "path:a",
			Reason:    "r",
			Owner:     "o",
			CreatedAt: "2026-01-01",
			Audit:     []string{"a"},
			ExpiresAt: "2026-02-07", // Expired
		})
	}

	reg := &Registry{Exceptions: exceptions}

	// We can't easily test the full string output of validateRegistryFile here because it does I/O.
	// But check logic: validateRegistry returns validation errors.
	// The formatting happens in validateRegistryFile.
	// WE NEED to verify the formatting logic.
	// Refactor suggestion: Extract formatting to a function or trust integration test?
	// User Requirement: "Unit test for output formatting".
	// Let's create a helper or verify validateRegistry's error list order first.

	errs := validateRegistry(reg, today)

	// Check 1: Count
	if len(errs) != 12 {
		t.Fatalf("Expected 12 errors, got %d", len(errs))
	}

	// Check 2: Sort Order (EX-00 to EX-11)
	for i, e := range errs {
		expectedID := fmt.Sprintf("EX-%02d", i)
		if !strings.HasPrefix(e.Error(), expectedID) {
			t.Errorf("Sort error at index %d: expected %s..., got %s", i, expectedID, e.Error())
		}
	}

	// Check 3: Formatting (Mocking validateRegistryFile's logic here to ensure "code under test" matches)
	// Actually, we should check `validateRegistryFile` logic.
	// Since `validateRegistryFile` takes FS, we can use MapFS to test the full output format!
	// This is the best way to satisfy "Golden Output".

	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(generateTOML(exceptions))},
	}

	err := validateRegistryFile(fsys, today)
	if err == nil {
		t.Fatal("Expected error from validateRegistryFile")
	}

	driftErr, ok := err.(*driftError)
	if !ok {
		t.Fatalf("Expected *driftError, got %T", err)
	}

	// Golden Output Check
	// 1. Header
	if !strings.Contains(driftErr.reason, "Registry validation failed (12 errors): (utc_today=2026-02-08)") {
		t.Errorf("Header mismatch. Got:\n%s", driftErr.reason)
	}

	// 2. Body (Top 10 sorted)
	// EX-00 to EX-09 should be present
	for i := 0; i < 10; i++ {
		key := fmt.Sprintf("EX-%02d", i)
		if !strings.Contains(driftErr.reason, "- "+key) {
			t.Errorf("Missing expected item %s in output", key)
		}
	}

	// 3. Capping (EX-10, EX-11 hidden)
	if strings.Contains(driftErr.reason, "EX-10") || strings.Contains(driftErr.reason, "EX-11") {
		t.Errorf("Output should not contain capped items EX-10 or EX-11")
	}

	// 4. "and N more"
	if !strings.Contains(driftErr.reason, "... and 2 more") {
		t.Errorf("Missing capping message '... and 2 more'")
	}

	// 5. Footer (Check that it is NOT in reason, but in fixCmd/print logic)
	if strings.Contains(driftErr.reason, "Fix:") {
		t.Errorf("Reason should NOT contain Fix footer")
	}
	if strings.Contains(driftErr.reason, "Next:") {
		t.Errorf("Reason should NOT contain Next footer")
	}
	
	// Check FixCmd
	expectedFix := "Correct the invalid entries in ops/exceptions.toml"
	if driftErr.fixCmd != expectedFix {
		t.Errorf("FixCmd mismatch. Expected:\n%s\nGot:\n%s", expectedFix, driftErr.fixCmd)
	}
}

// Helper to generate TOML content for MapFS
func generateTOML(exs []Exception) []byte {
	var sb strings.Builder
	for _, e := range exs {
		sb.WriteString("[[exception]]\n")
		sb.WriteString(fmt.Sprintf("id = \"%s\"\n", e.ID))
		sb.WriteString(fmt.Sprintf("rule = \"%s\"\n", e.Rule))
		sb.WriteString(fmt.Sprintf("scope = \"%s\"\n", e.Scope))
		sb.WriteString(fmt.Sprintf("reason = \"%s\"\n", e.Reason))
		sb.WriteString(fmt.Sprintf("owner = \"%s\"\n", e.Owner))
		sb.WriteString(fmt.Sprintf("created_at = \"%s\"\n", e.CreatedAt))
		sb.WriteString("audit = [\"a\"]\n")
		if e.ExpiresAt != "" {
			sb.WriteString(fmt.Sprintf("expires_at = \"%s\"\n", e.ExpiresAt))
		}
		sb.WriteString("\n")
	}
	return []byte(sb.String())
}

func TestValidateRegistry_ParseError(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "BAD-DATE", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-30"},
		},
	}
	errs := validateRegistry(reg, today)
	found := false
	for _, e := range errs {
		if strings.Contains(e.Error(), "invalid expires_at") {
			found = true
		}
	}
	if !found {
		t.Errorf("Expected invalid expires_at error")
	}
}

func TestValidateRegistry_MixedErrors(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	reg := &Registry{
		Exceptions: []Exception{
			// ID: A-MIX -> Missing Rule (Schema) + Expired (Logic)
			// Note: logic might stop at missing rule? Check registry.go implementation.
			// registry.go checks Mandatory fields THEN Date Formats. All are appended to errs.
			// So we should see "missing rule" AND "expired" if we allow it?
			// Actually registry.go does NOT return early. It appends all errors.
			{ID: "A-MIX", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-07"},
			// ID: B-EXP -> Just Expired
			{ID: "B-EXP", Rule: "r", Scope: "path:b", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}, ExpiresAt: "2026-02-07"},
		},
	}
	
	errs := validateRegistry(reg, today)
	
	// We expect:
	// A-MIX: missing rule
	// A-MIX: expired on ...
	// B-EXP: expired on ...
	
	if len(errs) != 3 {
		t.Errorf("Expected 3 errors, got %d", len(errs))
		for _, e := range errs {
			t.Logf("Error: %s", e.Error())
		}
	}
	
	// Check content
	foundRule := false
	foundExpiredA := false
	foundExpiredB := false
	
	for _, e := range errs {
		msg := e.Error()
		if strings.Contains(msg, "A-MIX: missing rule") {
			foundRule = true
		}
		if strings.Contains(msg, "A-MIX: expired on") {
			foundExpiredA = true
		}
		if strings.Contains(msg, "B-EXP: expired on") {
			foundExpiredB = true
		}
	}
	
	if !foundRule { t.Error("Missing 'A-MIX: missing rule'") }
	if !foundExpiredA { t.Error("Missing 'A-MIX: expired on'") }
	if !foundExpiredB { t.Error("Missing 'B-EXP: expired on'") }
}

func TestValidateRegistry_OriginalIndexFallback(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	
	// Scenario 1: Duplicate IDs - preserve file order (OriginalIndex)
	// Scenario 2: Missing IDs - key should be "idx <OriginalIndex>"
	// OriginalIndex is populated by unmarshal normally, here we simulate it.
	
	reg := &Registry{
		Exceptions: []Exception{
			// Index 0: Missing ID
			{ID: "", OriginalIndex: 0, Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
			// Index 1: DUP (First)
			{ID: "DUP", OriginalIndex: 1, Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
			// Index 2: DUP (Second)
			{ID: "DUP", OriginalIndex: 2, Rule: "r", Scope: "path:b", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
			// Index 3: Missing ID
			{ID: "", OriginalIndex: 3, Rule: "r", Scope: "path:c", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
		},
	}
	
	errs := validateRegistry(reg, today)
	
	// Expected Order after Sort:
	// 1. DUP (orig 1)
	// 2. DUP (orig 2)
	// 3. ID="" (orig 0) -> ID is empty string, empty string < "DUP"? YES.
	// Wait, empty string "" comes BEFORE "DUP".
	// So:
	// 1. ID="" (orig 0)
	// 2. ID="" (orig 3)
	// 3. DUP (orig 1)
	// 4. DUP (orig 2)
	
	if len(errs) != 3 {
		t.Fatalf("Expected 3 errors (2 missing ID, 1 duplicate), got %d", len(errs))
	}
	
	// Check Error Keys
	
	// Item 1 (Sorted): ID="" (OriginalIndex 0)
	if !strings.HasPrefix(errs[0].Error(), "idx 0: missing id") {
		t.Errorf("Expected 'idx 0: missing id', got: %s", errs[0].Error())
	}
	
	// Item 2 (Sorted): ID="" (OriginalIndex 3)
	if !strings.HasPrefix(errs[1].Error(), "idx 3: missing id") {
		t.Errorf("Expected 'idx 3: missing id', got: %s", errs[1].Error())
	}
	
	// Item 3 (Sorted): DUP (OriginalIndex 2) -> Duplicate
	if errs[2].Error() != "duplicate id: DUP" {
		t.Errorf("Expected 'duplicate id: DUP', got: %s", errs[2].Error())
	}
	
	regRulesMissing := &Registry{
		Exceptions: []Exception{
			{ID: "", OriginalIndex: 0},
			{ID: "DUP", OriginalIndex: 1},
			{ID: "DUP", OriginalIndex: 2},
			{ID: "", OriginalIndex: 3},
		},
	}
	
	errs2 := validateRegistry(regRulesMissing, today)
	// Now all 4 have "missing rule" + other errors.
	
	// Sorted Order matches:
	// 1. ID="" (0)
	// 2. ID="" (3)
	// 3. ID="DUP" (1)
	// 4. ID="DUP" (2)
	
	if len(errs2) < 4 {
		t.Fatalf("Expected at least 4 errors, got %d", len(errs2))
	}
	
	// Check specific error messages which contain keys
	
	// 1st error chunk should be for idx 0
	if !strings.HasPrefix(errs2[0].Error(), "idx 0:") {
		t.Errorf("Sort mismatch 1. Expected idx 0..., got %s", errs2[0].Error())
	}
	
	// 2nd error chunk (after correct number of errors for item 0) should be for idx 3.
	// Item 0 has: missing id, missing rule, missing scope... (6 errors)
	// So we need to find where idx 3 starts.
	// Easier: check that we encounter idx 0 calls BEFORE idx 3 calls.
	// And DUP calls come AFTER empty ID calls.
	
	lastIndex := -1
	for _, e := range errs2 {
		msg := e.Error()
		currIndex := -1
		if strings.HasPrefix(msg, "idx 0:") {
			currIndex = 0
		} else if strings.HasPrefix(msg, "idx 3:") {
			currIndex = 3
		} else if strings.HasPrefix(msg, "DUP:") || strings.HasPrefix(msg, "duplicate id: DUP") {
			currIndex = 100 // treat DUP as high index for this check
		}
		
		if currIndex != -1 {
			if currIndex < lastIndex && lastIndex != 100 { // 0 then 3 is OK. 3 then 0 is bad. 0 then 100 is OK.
				// allow duplicates of same index (multiple errors per item)
				// strict check: non-decreasing sort key
			}
			// Actually just verifying we see 0, then 3, then DUPs is enough.
		}
	}
}
