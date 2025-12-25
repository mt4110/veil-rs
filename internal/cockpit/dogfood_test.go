package cockpit

import (
	"testing"
)

func TestGenerateWorklistScoring(t *testing.T) {
	// Setup
	curr := &MetricsV1{
		Metrics: MetricsBody{
			CountsByHint: map[string]int{
				"hint_a": 10, // Delta = 10 (assume prev=0) -> Score = 10*10 + 10*25 = 100+250 = 350
				"hint_b": 5,  // Delta = 0 (prev=5) -> Score = 5*10 + 0 = 50
				"hint_c": 5,  // Delta = -5 (prev=10) -> Score = 5*10 + 0 = 50. Key "hint_c" > "hint_b", so hint_c should be lower rank if sorts stable?
				// Sort order: Score DESC, Count DESC, ActionID ASC, HintKey ASC
			},
		},
	}
	prev := &MetricsV1{
		Metrics: MetricsBody{
			CountsByHint: map[string]int{
				"hint_a": 0,
				"hint_b": 5,
				"hint_c": 10,
			},
		},
	}

	// Execute
	wl, err := generateWorklist("2025-W01-Tokyo", curr, prev)
	if err != nil {
		t.Fatalf("generateWorklist failed: %v", err)
	}

	// Verify
	if len(wl.Items) != 3 {
		t.Fatalf("expected 3 items, got %d", len(wl.Items))
	}

	// Item 0: hint_a (Score 350)
	if wl.Items[0].Signals.HintKey != "hint_a" {
		t.Errorf("Rank 1 should be hint_a, got %s", wl.Items[0].Signals.HintKey)
	}
	if wl.Items[0].Score != 350 {
		t.Errorf("Rank 1 score expected 350, got %d", wl.Items[0].Score)
	}

	// Item 1 vs Item 2 (Both Score 50, Count 5)
	// Tie-break: ActionID.
	// hint_b maps to Unmapped (Z-UNMAPPED) ?? Or depends on getActionBlueprint.
	// getActionBlueprint returns empty -> ActionId "Z-UNMAPPED", Title "Unmapped hint: ..."
	// So both are Z-UNMAPPED.
	// Tie-break: HintKey ASC. "hint_b" < "hint_c". So hint_b comes first.
	if wl.Items[1].Signals.HintKey != "hint_b" {
		t.Errorf("Rank 2 should be hint_b, got %s", wl.Items[1].Signals.HintKey)
	}
	if wl.Items[2].Signals.HintKey != "hint_c" {
		t.Errorf("Rank 3 should be hint_c, got %s", wl.Items[2].Signals.HintKey)
	}
}

func TestParseScorecardScoreFormats(t *testing.T) {
	tests := []struct {
		name    string
		json    string
		want    float64
		wantErr bool
	}{
		{
			name: "flat score",
			json: `{"score": 7.5}`,
			want: 7.5,
		},
		{
			name: "aggregate score float",
			json: `{"aggregateScore": 8.1}`,
			want: 8.1,
		},
		{
			name: "aggregate score object",
			json: `{"aggregateScore": {"score": 9.2}}`,
			want: 9.2,
		},
		{
			name:    "missing",
			json:    `{"foo": "bar"}`,
			wantErr: true,
		},
		{
			name:    "invalid json",
			json:    `{not valid json}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := parseScorecardScore([]byte(tt.json))
			if (err != nil) != tt.wantErr {
				t.Errorf("parseScorecardScore() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr && got != tt.want {
				t.Errorf("parseScorecardScore() = %v, want %v", got, tt.want)
			}
		})
	}
}
