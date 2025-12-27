package signal

import (
	"testing"
)

func TestDecWeek(t *testing.T) {
	tests := []struct {
		input string
		want  string
	}{
		{"2025-W12", "2025-W11"},
		{"2025-W02", "2025-W01"},
		{"2025-W01", "2024-W52"}, // This depends on the year; 2024 has 52 weeks?
		// 2024 is leap year starting Monday? 
		// 2024-12-31 is Tuesday. 
		// ISO week 52 of 2024 ends Sun Dec 29. Wait.
		// Go ISOWeek logic is robust. Let's trust it but verify expected output for known years.
		// 2025-01-01 is Wednesday. Week 1 of 2025 starts Dec 30 2024.
		// So 2024-W52 is the week before?
		{"2024-W01", "2023-W52"}, 
	}
	
	for _, tt := range tests {
		got := decWeek(tt.input)
		if got != tt.want {
			t.Errorf("decWeek(%q) = %q, want %q", tt.input, got, tt.want)
		}
	}
}

func TestCheckRecurrenceCriteria(t *testing.T) {
	// Rule: 3 consecutive OR 3 out of 4
	tests := []struct {
		name    string
		weeks   []string
		current string
		want    bool
	}{
		{
			"Empty",
			[]string{},
			"2025-W10",
			false,
		},
		{
			"Only Current (1)",
			[]string{"2025-W10"},
			"2025-W10",
			false,
		},
		{
			"Two Consecutive (2)",
			[]string{"2025-W09", "2025-W10"},
			"2025-W10",
			false,
		},
		{
			"Three Consecutive (3) - HIT",
			[]string{"2025-W08", "2025-W09", "2025-W10"},
			"2025-W10",
			true,
		},
		{
			"Gap but 3 of 4 (W07, W09, W10) - Missing W08",
			// Window [07, 08, 09, 10]
			// Hits: W07(yes), W09(yes), W10(yes) -> 3 hits.
			[]string{"2025-W07", "2025-W09", "2025-W10"},
			"2025-W10",
			true,
		},
		{
			"Gap too wide (W06, W09, W10) - Missing W07, W08",
			// Window [07, 08, 09, 10]
			// Hits: W09, W10. (W06 is outside). -> 2 hits.
			[]string{"2025-W06", "2025-W09", "2025-W10"},
			"2025-W10",
			false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := checkRecurrenceCriteria(tt.weeks, tt.current); got != tt.want {
				t.Errorf("checkRecurrenceCriteria() = %v, want %v", got, tt.want)
			}
		})
	}
}
