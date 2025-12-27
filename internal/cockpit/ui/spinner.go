package ui

import (
	"fmt"
	"os"
	"time"
)

// Spinner provides a simple TTY spinner that is silent in CI.
type Spinner struct {
	msg       string
	active    bool
	stop      chan struct{}
	done      chan struct{}
	startTime time.Time
}

// NewSpinner creates a new spinner with the given message.
func NewSpinner(msg string) *Spinner {
	return &Spinner{
		msg:    msg,
		stop:   make(chan struct{}),
		done:   make(chan struct{}),
		active: shouldSpin(),
	}
}

// Start begins the spinner animation if enabled.
func (s *Spinner) Start() {
	if !s.active {
		// In CI/non-interactive, maybe print a static log?
		// "Compiling... "
		// User requirement 3: "In CI... static log only"
		// But let's check if the user wants "Starting..." log here or just silence until Done?
		// "3. In CI... regular println log only"
		// Usually this means "Starting phase X..."
		fmt.Printf("=> %s...\n", s.msg)
		return
	}

	s.startTime = time.Now()
	go s.run()
}

// StopOK stops the spinner with a success message.
func (s *Spinner) StopOK(okMsg string) {
	if !s.active {
		fmt.Printf("   ✓ %s\n", okMsg)
		return
	}
	s.stop <- struct{}{}
	<-s.done
	// Overwrite spinner line
	fmt.Printf("\r\033[K✓ %s (%v)\n", okMsg, time.Since(s.startTime).Round(time.Millisecond))
}

// StopWarn stops the spinner with a warning message.
func (s *Spinner) StopWarn(warnMsg string) {
	if !s.active {
		fmt.Printf("   ⚠ %s\n", warnMsg)
		return
	}
	s.stop <- struct{}{}
	<-s.done
	fmt.Printf("\r\033[K⚠ %s\n", warnMsg)
}

func (s *Spinner) run() {
	// Simple braille spinner
	chars := []rune("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	i := 0
	for {
		select {
		case <-s.stop:
			close(s.done)
			return
		case <-ticker.C:
			i = (i + 1) % len(chars)
			// \r to return to start, \033[K to clear line (just in case)
			fmt.Printf("\r%c %s...", chars[i], s.msg)
		}
	}
}

// shouldSpin returns true only if we are in a local TTY and explicit disable flags are unset.
func shouldSpin() bool {
	// 1. Explicit Disable
	if os.Getenv("VEIL_NO_SPINNER") == "1" {
		return false
	}

	// 2. CI Detection
	if os.Getenv("CI") != "" || os.Getenv("GITHUB_ACTIONS") == "true" {
		return false
	}

	// 3. Dumb Terminal
	if os.Getenv("TERM") == "dumb" {
		return false
	}

	// 4. (Optional) Check file descriptor if strict TTY check needed (needs 'golang.org/x/term' or syscall)
	// User said "dependency free check is ideal, but ENV base is OK".
	// We will stick to ENV base for now as per requirement.
	return true
}
