package cockpit

import (
	"strconv"
	"strings"
)

func isValidWeekID(weekID string) bool {
	// Format: YYYY-Www (ISO week), e.g. 2025-W52
	parts := strings.Split(weekID, "-")
	if len(parts) != 2 {
		return false
	}
	// YYYY
	if len(parts[0]) != 4 {
		return false
	}
	if _, err := strconv.Atoi(parts[0]); err != nil {
		return false
	}
	// Www
	w := parts[1]
	if len(w) != 3 || !strings.HasPrefix(w, "W") {
		return false
	}
	wn, err := strconv.Atoi(w[1:])
	if err != nil {
		return false
	}
	if wn < 1 || wn > 53 {
		return false
	}
	return true
}
