package main

type ErrorCode string

const (
	E_ORDER    ErrorCode = "E_ORDER"
	E_IDENTITY ErrorCode = "E_IDENTITY"
	E_PAX      ErrorCode = "E_PAX"
	E_XATTR    ErrorCode = "E_XATTR"
	E_TIME     ErrorCode = "E_TIME"
	E_GZIP     ErrorCode = "E_GZIP"
	E_PATH     ErrorCode = "E_PATH"
	E_TYPE     ErrorCode = "E_TYPE"
	E_LAYOUT   ErrorCode = "E_LAYOUT"
	E_SHA256   ErrorCode = "E_SHA256"
	E_SEAL     ErrorCode = "E_SEAL"
	E_MISSING  ErrorCode = "E_MISSING"
	E_EXTRA    ErrorCode = "E_EXTRA"
	E_BUDGET   ErrorCode = "E_BUDGET"
	E_EVIDENCE ErrorCode = "E_EVIDENCE"
	E_CONTRACT ErrorCode = "E_CONTRACT"
)

func (e ErrorCode) String() string {
	return string(e)
}

type VError struct {
	Code   ErrorCode
	Reason string
	Path   string
	Detail string
}

func (e *VError) Error() string {
	if e.Path == "" {
		return e.Code.String() + " " + e.Detail
	}
	return e.Code.String() + " " + e.Path + " " + e.Detail
}

func (e *VError) WithReason(reason string) *VError {
	e.Reason = reason
	return e
}

func (e *VError) Line() string {
	reason := e.Reason
	if reason == "" {
		reason = string(e.Code)
	}

	line := "ERROR: " + reason
	if e.Path != "" {
		line += " path=" + e.Path
	}
	if e.Detail != "" {
		line += " detail=" + e.Detail
	}
	line += " stop=1"
	return line
}

func NewVError(code ErrorCode, path string, detail string) *VError {
	return &VError{
		Code:   code,
		Path:   path,
		Detail: detail,
	}
}

func WrapVError(code ErrorCode, path string, err error) *VError {
	return &VError{
		Code:   code,
		Path:   path,
		Detail: err.Error(),
	}
}
