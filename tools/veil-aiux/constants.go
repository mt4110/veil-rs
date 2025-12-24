package main

const (
	E_OK    = 0
	E_USAGE = 2
	E_CHECK = 3
	E_GEN   = 4
	E_IO    = 5
)

const (
	maxAIPackDiffBytes = 2 * 1024 * 1024 // 2MB
	maxAIPackFiles     = 50

	shortTemplateMissing = "Template missing"
	shortInvalidArgs     = "Invalid arguments"
	shortUnexpectedArgs  = "Unexpected arguments"
	shortInvalidVersion  = "Invalid version"

	fmtUnexpectedArgs = "unexpected args: %v"
	gitDiffHeadSuffix = "...HEAD"

	msgExampleVersion = "Example: --version v0.14.0"

	shortBadTemplate        = "Bad template"
	fixBadTemplate          = "Fix template content (non-empty, exactly 1 H1, even code fences)"
	shortWriteFailed        = "Write failed"
	fixCheckPermissions     = "Check disk permissions"
	fixRemoveExtraArgs      = "Remove extra args or use --help"
	shortDistViolated       = "Dist contract violated"
	shortReadTemplateFailed = "Failed to read template"
	fixCheckFilePermissions = "Check file permissions"

	gitNoExtDiff = "--no-ext-diff"
)
