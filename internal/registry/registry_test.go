package registry

import (
	"sort"
	"testing"
	"time"
)

func parseDate(t *testing.T, s string) time.Time {
	d, err := time.Parse("2006-01-02", s)
	if err != nil {
		t.Fatalf("failed to parse date %s: %v", s, err)
	}
	return d
}

func TestCalculateStatus(t *testing.T) {
	today := parseDate(t, "2026-02-08")

	tests := []struct {
		name      string
		expiresAt string
		want      Status
	}{
		{
			name:      "No Expiry",
			expiresAt: "",
			want:      StatusActive,
		},
		{
			name:      "Future (Active)",
			expiresAt: "2026-03-01", // > 7 days
			want:      StatusActive,
		},
		{
			name:      "Expiring Soon (Boundary 7 days)",
			expiresAt: "2026-02-15", // 7 days from today
			want:      StatusExpiringSoon,
		},
		{
			name:      "Expiring Soon (Tomorrow)",
			expiresAt: "2026-02-09",
			want:      StatusExpiringSoon,
		},
		{
			name:      "Today (Active? No, Today is valid, so not expired, but is it expiring soon?)", 
			// Today is 0 days away. <= 7 days. So Expiring Soon.
			expiresAt: "2026-02-08",
			want:      StatusExpiringSoon,
		},
		{
			name:      "Expired (Yesterday)",
			expiresAt: "2026-02-07",
			want:      StatusExpired,
		},
		{
			name:      "Invalid Format",
			expiresAt: "invalid",
			want:      StatusActive, // Fallback
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ex := &Exception{ExpiresAt: tt.expiresAt}
			got := CalculateStatus(ex, today)
			if got != tt.want {
				t.Errorf("CalculateStatus() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSortDeterminism(t *testing.T) {
	// Mixed IDs and missing IDs to test primary/secondary sort key
	// Original indices: 0, 1, 2, 3, 4
	
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "B", OriginalIndex: 0},
			{ID: "A", OriginalIndex: 1},
			{ID: "", OriginalIndex: 2},
			{ID: "A", OriginalIndex: 3},
			{ID: "", OriginalIndex: 4},
		},
	}
	
	// Expected Order:
	// 1. ID="" (Idx 2)
	// 2. ID="" (Idx 4)
	// 3. ID="A" (Idx 1)
	// 4. ID="A" (Idx 3)
	// 5. ID="B" (Idx 0)

	sort.Slice(reg.Exceptions, func(i, j int) bool {
		if reg.Exceptions[i].ID == reg.Exceptions[j].ID {
			return reg.Exceptions[i].OriginalIndex < reg.Exceptions[j].OriginalIndex
		}
		return reg.Exceptions[i].ID < reg.Exceptions[j].ID
	})
	
	expectedIndices := []int{2, 4, 1, 3, 0}
	
	for i, ex := range reg.Exceptions {
		if ex.OriginalIndex != expectedIndices[i] {
			t.Errorf("Index %d: want OriginalIndex %d, got %d (ID=%q)", i, expectedIndices[i], ex.OriginalIndex, ex.ID)
		}
	}
}
