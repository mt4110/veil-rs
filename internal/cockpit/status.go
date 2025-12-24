package cockpit

func Status() (string, error) {
	g := GitX{}
	return g.Run("status", "-sb")
}
