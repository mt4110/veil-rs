package signal

import (
	"encoding/json"
	"os"
)

func LoadLedger(path string) (*Ledger, error) {
	data, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return &Ledger{V: 1, Signals: []Recurrence{}}, nil
	}
	if err != nil {
		return nil, err
	}

	var l Ledger
	if err := json.Unmarshal(data, &l); err != nil {
		return nil, err
	}
	return &l, nil
}

func SaveLedger(path string, l *Ledger) error {
	data, err := json.MarshalIndent(l, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, append(data, '\n'), 0644)
}

func SaveSignalsV1(path string, signals []Signal) error {
	data, err := json.MarshalIndent(signals, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, append(data, '\n'), 0644)
}
