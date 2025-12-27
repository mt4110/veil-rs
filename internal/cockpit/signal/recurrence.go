package signal

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
	"time"
)

// DetectRecurrence updates the Ledger based on current Signals.
// It applies the "3 consecutive" or "3 of 4" rules.
func DetectRecurrence(ledger *Ledger, currentSignals []Signal, currentWeekID string) {
	// 1. Mark all existing as candidate for update
	recMap := make(map[string]*Recurrence)
	for i := range ledger.Signals {
		r := &ledger.Signals[i]
		key := r.Category + ":" + r.CauseTag
		recMap[key] = r
	}

	// 2. Process Current Signals
	for _, s := range currentSignals {
		key := s.Category + ":" + s.CauseTag
		
		var rec *Recurrence
		if r, exists := recMap[key]; exists {
			rec = r
		} else {
			// New Candidate
			newRec := Recurrence{
				Category:     s.Category,
				CauseTag:     s.CauseTag,
				Status:       "inactive", // Pending criteria
				FirstSeenWeek: currentWeekID,
				LastSeenWeek:  "",
				HitCount:      0,
				RecentWeeks:   []string{},
			}
			rec = &newRec
		}

		// Update Stats
		rec.LastSeenWeek = currentWeekID
		rec.HitCount++
		
		if !contains(rec.RecentWeeks, currentWeekID) {
			rec.RecentWeeks = append(rec.RecentWeeks, currentWeekID)
		}
		
		// Trim RecentWeeks (keep last ~8 for context)
		if len(rec.RecentWeeks) > 8 {
			rec.RecentWeeks = rec.RecentWeeks[len(rec.RecentWeeks)-8:]
		}

		// Evaluate Logic
		if checkRecurrenceCriteria(rec.RecentWeeks, currentWeekID) {
			rec.Status = "active"
		}
		
		recMap[key] = rec
	}

	// 3. Reconstruct Ledger List
	var newList []Recurrence
	var keys []string
	for k := range recMap {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	
	for _, k := range keys {
		newList = append(newList, *recMap[k])
	}
	
	ledger.Signals = newList
}

func checkRecurrenceCriteria(weeks []string, current string) bool {
	if len(weeks) < 3 {
		return false
	}
	sort.Strings(weeks)
	
	// Robust "3 of 4" logic by counting "weeks back from current".
	return countRecentHits(weeks, current, 4) >= 3
}

func countRecentHits(weeks []string, current string, windowSize int) int {
	hits := 0
	candidates := getPrecedingWeeks(current, windowSize) // [W12, W11, W10, W09]
	
	for _, c := range candidates {
		if contains(weeks, c) {
			hits++
		}
	}
	return hits
}

func getPrecedingWeeks(current string, count int) []string {
	var res []string
	curr := current
	for i := 0; i < count; i++ {
		res = append(res, curr)
		curr = decWeek(curr)
	}
	return res
}

func decWeek(w string) string {
	parts := strings.Split(w, "-W")
	if len(parts) != 2 {
		return w
	}
	y, err := strconv.Atoi(parts[0])
	if err != nil { return w }
	wn, err := strconv.Atoi(parts[1])
	if err != nil { return w }

	// If wn > 1, simple decrement.
	if wn > 1 {
		return fmt.Sprintf("%04d-W%02d", y, wn-1)
	}
	
	// Rollback year
	// Find Dec 28th of prev year (always in last week of that year)
	prevY := y - 1
	t := time.Date(prevY, time.December, 28, 0, 0, 0, 0, time.UTC)
	py, pw := t.ISOWeek()
	return fmt.Sprintf("%04d-W%02d", py, pw)
}
