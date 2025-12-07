# Security Review Notes

## Overview
This document outlines the security considerations, threat model, and mitigation strategies for `veil-rs`.

## Threat Model

### 1. False Negatives (Missed Secrets)
- **Risk**: High. Secrets leaked into git history.
- **Mitigation**:
    - "Gatchigachi" strict defaults (Phase 7).
    - Multi-layered Scoring (Context + Base Severity).
    - Japan-specific optimizations (MyNumber Check Digit).

### 2. False Positives (Noise)
- **Risk**: Medium. Developer fatigue, ignoring warnings.
- **Mitigation**:
    - `veil ignore` command for easy allowlisting.
    - Context-aware scoring (e.g., test files have lower score).

### 3. Performance DoS (Denial of Service)
- **Risk**: Low (Local tool). Scanning massive binaries freezes CI.
- **Mitigation**:
    - `MAX_FILE_SIZE` limit (1MB default).
    - Parallel scanning with `rayon`.

## Future Security Roadmap
- **Git History Scanning**: Uncover historical leaks.
- **Secret Validity Checks**: Verify AWS keys/Slack tokens against APIs (Optional/Opt-in).
- **Signed Commits**: Integrate with GPG/SSH signing flows.
