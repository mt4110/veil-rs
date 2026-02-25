package main

import (
	"encoding/json"
	"fmt"
	"io"
	"strings"
)

func ParseContractJSON(b []byte) (*Contract, error) {
	var c Contract
	if err := json.Unmarshal(b, &c); err != nil {
		return nil, WrapVError(E_CONTRACT, "contract.json", err)
	}
	return &c, nil
}

func ParseContract(r io.Reader) (*Contract, error) {
	b, err := io.ReadAll(r)
	if err != nil {
		return nil, WrapVError(E_CONTRACT, "contract.json", err)
	}
	return ParseContractJSON(b)
}

func ValidateContractV11(c *Contract) error {
	if c.ContractVersion != "1.1" {
		return NewVError(E_CONTRACT, "contract.json", fmt.Sprintf("unsupported version: %s (want 1.1)", c.ContractVersion)).WithReason("contract_version_unsupported")
	}
	if c.Mode != "strict" && c.Mode != "wip" {
		return NewVError(E_CONTRACT, "contract.json", fmt.Sprintf("invalid mode: %s", c.Mode)).WithReason("contract_invalid")
	}
	if c.EpochSec <= 0 {
		return NewVError(E_CONTRACT, "contract.json", "invalid epoch_sec").WithReason("contract_invalid")
	}
	if len(c.HeadSHA) != 40 {
		return NewVError(E_CONTRACT, "contract.json", "invalid head_sha (want 40 hex chars)").WithReason("contract_invalid")
	}
	if c.Evidence.Required {
		if !strings.HasPrefix(c.Evidence.PathPrefix, "review/evidence/") || !strings.HasSuffix(c.Evidence.PathPrefix, "/") {
			return NewVError(E_CONTRACT, "contract.json", "invalid evidence.path_prefix").WithReason("contract_invalid")
		}
	}
	return nil
}
