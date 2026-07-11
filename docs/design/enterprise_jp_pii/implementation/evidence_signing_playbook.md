# Evidence Signing Playbook

This playbook defines the v1 operating model for signing Veil Evidence Packs in Japanese
enterprise deployments. It is intentionally a runbook, not cryptographic implementation code.

## Purpose

Evidence signing gives audit teams a stable way to bind an exported `evidence.zip` to an approval
record without changing the Evidence Pack v1 ZIP contract.

The signed object is the raw bytes SHA256 of `run_meta.json`, not the ZIP file and not a normalized
JSON representation.

## Non-Goals

- Do not add signature files inside `evidence.zip` in v1.
- Do not make "tamper-proof" claims. Use tamper-evident wording only.
- Do not require network access, transparency logs, or SaaS identity providers.
- Do not sign raw findings, source snippets, tokens, or Authorization headers.
- Do not replace `veil verify`; signing is an approval layer above structural verification.

## Required Inputs

| Input | Source |
|---|---|
| Evidence ZIP | Local Audit UI or CLI export |
| `run_meta.json` raw bytes SHA256 | `veil verify --expect-run-meta-sha256` anchor or locally extracted raw bytes |
| Run owner | Person or team accountable for the scan |
| Repository identity | Repository URL or internal project ID |
| Git revision | Commit SHA, release tag, or source snapshot ID |
| Verification command | Exact `veil verify ...` command and result |
| Approval record | Ticket, change request, audit ledger, or internal approval document |

## Signing Boundary

The approval signature must cover a small manifest outside the ZIP:

```json
{
  "schemaVersion": "veil-evidence-signing-playbook-v1",
  "evidence": {
    "runMetaSha256": "hex-encoded-sha256",
    "zipFilename": "evidence.zip"
  },
  "source": {
    "repository": "https://example.invalid/org/repo",
    "revision": "commit-or-release-id"
  },
  "verification": {
    "command": "veil verify evidence.zip --expect-run-meta-sha256 <hash> --require-complete",
    "status": "pass"
  },
  "approval": {
    "ticket": "SEC-1234",
    "approvedBy": "security-team",
    "approvedAt": "2026-07-11T00:00:00Z"
  }
}
```

This manifest may be signed with an enterprise-approved detached signature mechanism, such as an
internal PKI, GPG, hardware-backed signing key, or an offline approval ledger. The specific signing
tool is not part of the v1 Veil contract.

## Procedure

1. Generate `evidence.zip` locally.
2. Run `veil verify evidence.zip --require-complete`.
3. Extract or record the raw bytes SHA256 for `run_meta.json`.
4. Re-run verify with `--expect-run-meta-sha256 <hash>`.
5. Create the signing manifest from the required inputs.
6. Sign the manifest outside `evidence.zip`.
7. Store the manifest, detached signature, and verification output in the approval record.
8. Keep `evidence.zip` immutable after approval. If findings, config, baseline, or source revision
   change, generate a new Evidence Pack and repeat the process.

## Failure Handling

| Failure | Action |
|---|---|
| `veil verify` exits 2 | Reject the artifact. Regenerate Evidence after fixing structural or integrity issues. |
| `veil verify` exits 1 | Treat as policy failure or incomplete scan. Escalate according to the repository policy. |
| `run_meta.json` hash differs from the signed manifest | Reject as a different artifact. Do not override manually. |
| Signature validation fails | Reject the approval record. Re-sign only after identity and key state are checked. |
| Signing key is rotated or revoked | Keep old approvals verifiable through the enterprise key-retention policy. |

## Retention

Retain these together:

- `evidence.zip`
- signing manifest
- detached signature or approval ledger entry
- `veil verify` output
- approval ticket or audit record

The retention period is owned by the adopting organization. Veil v1 only requires that the
`run_meta.json` hash and verification result remain recoverable from the retained record.

## Implementation Notes

- Future native signature verification should verify this external manifest first, then call
  the existing Evidence verifier.
- Remote RulePack signatures are a separate trust boundary and are not covered by this playbook.
- Evidence signing must remain local-first. Any enterprise network integration is opt-in and must
  not upload source, PII, findings, or Evidence contents.
