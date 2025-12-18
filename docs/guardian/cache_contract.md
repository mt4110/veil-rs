# Guardian Cache Contract (OSV) — Cache Constitution (v0.12, veil-rs aligned)

> **Status:** Draft (v0.12)
> **Modules:** `providers::osv::{client, details_store}` / `guardian_next::{cache, outcome, error}`
> **Primary types:** `OsvClient`, `DetailsStore`, `FetchOutcome`
> **Prime directive:** Predictability > convenience. Cache must not be a haunted house.

---

## 1. Purpose

This document defines the **normative contract** for OSV cache behavior in Guardian Next:

* **Format contract** (layout + schema) that is stable across releases.
* **Migration strategy** from legacy → `v1/` that converges naturally (migrate-on-read).
* **Deterministic resolution** under real-world failures (corruption, offline, partial writes).
* A clear **separation of responsibilities** between `DetailsStore` (storage law) and `OsvClient` (network + policy).

---

## 2. Non-Goals (v0.12)

* Remote/shared/distributed cache
* External metric sinks (Prometheus/OTEL exporters)
* Cache encryption/compression
* Cross-machine coherence guarantees

---

## 3. Terminology (veil-rs aligned)

### 3.1 Directories

* **Cache Root:** `<cache_dir>/osv`
* **Query Cache:** `<cache_dir>/osv/query/`
* **v1 Cache Dir:** `<cache_dir>/osv/vulns/v1/` (primary, strict)
* **Legacy Dir:** `<cache_dir>/osv/` (fallback root)

### 3.2 Storage outcomes (DetailsStore-level)

* **V1 Hit:** v1 envelope read & validated
* **Legacy Hit:** legacy read & interpreted as valid payload
* **Miss:** not found OR treated unusable (after quarantine/unsupported)
* **Corrupt:** parse/validation failed
* **Unsupported:** parse ok but unknown `schema_version`

### 3.3 Client outcomes (FetchOutcome-level)

`FetchOutcome` is the canonical end-user / report-facing result from `OsvClient`.

**Normative mapping (recommended):**

* `FetchOutcome::CacheHitFresh` — cache satisfied request (fresh)
* `FetchOutcome::CacheHitStale` — cache satisfied request (stale)
* `FetchOutcome::HitLegacyMigrated` — legacy satisfied request and migrated to v1
* `FetchOutcome::Fetched` — network fetch performed and v1 written
* `FetchOutcome::Error` — resolution failed (only after all legal paths exhausted)

---

## 4. Directory Layout & Conflict Handling

### 4.1 Required Layout

* Root MUST be `<cache_dir>/osv`
* Details v1 MUST be `<cache_dir>/osv/vulns/v1/`
* Query Cache SHOULD be `<cache_dir>/osv/query/`

### 4.2 Conflict: vulns/v1 exists as a file

If `<cache_dir>/osv/vulns/v1` is a file, not a directory:

1. `DetailsStore` MUST quarantine it (implementation: `v1.corrupt_dirs_conflict.<ts>.<uniq>`)
2. `DetailsStore` MUST create directory `<cache_dir>/osv/vulns/v1/`

**Rationale:** Layout must be deterministic across upgrades.

---

## 5. Key Normalization & File Naming

### 5.1 Normalized Key (NormKey)

* Allowed chars: `[A-Za-z0-9._-]`
* All other chars MUST be sanitized deterministically (implementation: unsafe chars → `_`)
* If sanitization OR truncation occurs, a collision-avoid suffix hash MUST be appended.
* If the original key is already safe and length ≤ 128, the hash suffix MAY be omitted (NormKey == original).



**Concept (implementation-aligned):**
- If unchanged and length ≤ 128: `NormKey = <original>`
- Otherwise: `NormKey = <sanitized_prefix64>-<blake3_16hex>`

Where:
- `<sanitized_prefix64>` is sanitized and truncated to 64 chars.
- `<blake3_16hex>` is the first 16 hex chars of `blake3(original_key)`.

### 5.2 v1 File Path

`v1_path = <cache_dir>/osv/vulns/v1/<NormKey>.json`

**Example:**
- `GHSA-1234` -> `< ... >/GHSA-1234.json`
- `GHSA:Bad/Key` -> `< ... >/GHSA_Bad_Key-a1b2c3d4e5f60708.json` (Sanitized + blake3_16hex)

Legacy paths follow historical behavior (see §8).

---

## 6. Envelope Schema (Schema v1)

### 6.1 Required JSON shape

A v1 cache file MUST be a JSON object with (at minimum) these required fields:

* `schema_version`: `1`
* `key`: original key (typically the vuln_id string)
* `created_at_unix`: Unix seconds (u64)
* `fetched_at_unix`: Unix seconds (u64)
* `source`: `"fetch"` | `"legacy_migration"`
* `payload`: OSV payload object

Optional fields (serialized only when present in v0.12):

* `expires_at_unix`: Unix seconds (u64)
* `etag`: HTTP ETag string



Example (typical v0.12 write with `expires_at_unix = None`):

```json
{
  "schema_version": 1,
  "key": "original:key",
  "created_at_unix": 1700000000,
  "fetched_at_unix": 1700000000,
  "etag": "W/\"tag\"",
  "source": "fetch",
  "payload": { ... }
}
```

### 6.2 Field semantics (normative)

* `created_at_unix`: this v1 record was written (Unix Seconds).
* `fetched_at_unix`: payload was obtained from network (Unix Seconds). Used for Freshness calc via TTL.
* `expires_at_unix`: RESERVED/OPTIONAL. Explicit expiration (Unix Seconds). Current implementation determines Fresh/Stale/Expired using `fetched_at_unix` + CachePolicy (TTL + grace).
* `etag`: HTTP ETag from upstream (if available). MUST be persisted for 304 optimization.

### 6.3 Validation rules (normative)

A v1 entry is invalid (treated as **Corrupt**) if:

* JSON parse fails
* required fields missing / wrong types
* timestamps not valid u64
* `source` not in allowed set
* `expires_at_unix` is present AND `expires_at_unix < fetched_at_unix` (contract violation)

**Notes:**
* `expires_at_unix` MAY be null or omitted in v0.12.
* If omitted, the entry is still valid; freshness is derived from `fetched_at_unix` + TTL.

An entry is **Unsupported** if:

* JSON parse succeeds
* but `schema_version` is not recognized (!= 1)

**Unsupported** is not “corrupt” — but still must not block resolution.

---

## 7. Quarantine Rules (Corrupt / Unsupported)

### 7.1 Principle

Cache MUST NOT be silently deleted.
`DetailsStore` MUST quarantine Corrupt/Unsupported files by renaming.

### 7.2 Naming (normative)

Quarantine suffix MUST ensure uniqueness under concurrency:

Pattern: `.<reason>.<ts>.<uniq>`

* `reason`: `corrupt` or `unsupported_vN`
* `ts`: epoch or compact timestamp
* `uniq`: pid / counter / random

Examples:
* `abc-deadbeef.json.corrupt.1730000000.12345`
* `abc-deadbeef.json.unsupported_v2.1730000001.2`

### 7.3 Unsupported handling (normative)

If `schema_version` is unknown:

* MUST quarantine as `unsupported_vN`
* MUST be treated as **Miss** for resolution (continue to legacy/fetch)

---

## 8. Legacy Compatibility (Fallback Read)

### 8.1 Legacy is fallback read only

v0.12 MUST be able to read legacy cache entries as fallback to avoid breaking upgrades.

### 8.2 Legacy write policy

v0.12 MUST NOT write new legacy entries.
All writes MUST go to v1 envelope.

### 8.3 Legacy validity

If legacy content can be interpreted into a valid OSV payload → **Valid Legacy**
Otherwise → **Legacy Corrupt** (and treated like miss for progression)

---

## 9. Migration Policy (Migrate-on-Read)

### 9.1 Core rule (normative)

If legacy is a Valid Legacy hit:

1. `DetailsStore` MUST construct a v1 envelope
2. MUST write v1 atomically
3. MUST return the legacy-derived payload as the result

This is **migrate-on-read** and must converge migration naturally.

### 9.2 Timestamp mapping from legacy (normative)

When migrating legacy → v1:

* If legacy provides reliable timing metadata:
    * preserve into `fetched_at` and `expires_at` accordingly
* If legacy has no reliable timing metadata:
    * MUST be conservative to avoid “accidentally freshening old data”

**Rule A (Normative for v0.12, implementation-aligned)**:
If legacy timing is unknown / unreliable:
* `created_at_unix = now`
* `fetched_at_unix = now - (fresh_ttl_secs + 1)` (forces Stale; MUST NOT appear Fresh)
* `expires_at_unix = null`
* `source = "legacy_migration"`

Rationale: Avoid “freshening” old legacy data while still allowing offline fallback.

---

## 10. Resolution Law (The Law) — v1 → legacy → fetch

This section is **the law**. `DetailsStore` and `OsvClient` MUST implement behavior consistent with it. Tests MUST encode it.

### 10.1 Deterministic read priority (normative)

**Step 1: Read v1**
* If v1 is Valid → return (Fresh/Stale by `fetched_at_unix` + TTL)
* If v1 is Corrupt → quarantine → continue
* If v1 is Unsupported → quarantine → continue
* If v1 Miss → continue

Fresh/Stale/Expired MUST be computed from `fetched_at_unix` + CachePolicy (TTL + grace). `expires_at_unix` is not required in v0.12.

**Step 2: Read legacy**
* If legacy is Valid → migrate-on-read (write v1) → return
* If legacy Miss/Corrupt → continue

**Step 3: Network fetch**
* If online → fetch → write v1 envelope → return
* If offline → error (only if both v1 and legacy failed)

### 10.2 Offline constraint (normative)

Offline MUST NOT fail if legacy can satisfy the request.
Offline MAY fail only when:
* v1 unusable (miss/corrupt/unsupported) AND
* legacy unusable (miss/corrupt)

---

## 11. Responsibility Split (implementation guardrails)

### 11.1 DetailsStore responsibilities (storage law)
* Layout validation (ensure v1 dir; resolve v1-file conflict)
* Key normalization + path mapping
* Load/validate v1 envelope
* Legacy read + interpretation
* Quarantine operations
* Migrate-on-read write (legacy → v1)
* Save v1 envelope (atomic)

### 11.2 OsvClient responsibilities (network + policy)
* Determine online/offline mode (strict)
* Execute fetch logic (retry, budgets, concurrency gate)
* Convert store outcomes into `FetchOutcome`
* Emit metrics (Task 2) and report surfaces

---

## 12. Pseudocode (Directly Implementable)

### 12.1 DetailsStore API (normative shape)

```rust
// Types (conceptual)
struct QuarantineFlags {
    corrupt: bool,
    unsupported: bool,
    conflict: bool,
}

// FIXED: Synced with implementation (was StoreRead, added QuarantineFlags)
enum StoreLoad {
    Hit {
        entry: EntryV1,
        source: StoreSource, // V1 | Legacy
        migrated: bool,
        quarantined: QuarantineFlags,
    },
    Miss {
        quarantined: QuarantineFlags,
    },
}

// Functions
fn ensure_v1_dir() -> Result<()>;
fn load(key) -> StoreLoad; // No Result, internal errors -> Miss/Quarantine
fn save_v1_envelope(key, envelope) -> Result<()>;
fn migrate_legacy_to_v1(key, legacy_payload, meta) -> Result<EntryV1>;
```

### 12.2 DetailsStore::load (The Law encoded)

**Contract:**
* This function MUST NOT perform network access.
* It only resolves from v1 and legacy, and can migrate legacy → v1.
* It MUST report quarantine events even on Hit (e.g. v1 corrupt -> legacy hit).

**Algorithm:**
1. Initialize `flags = QuarantineFlags::empty()`
2. `ensure_v1_dir()` (if fail: record conflict flag)
3. Try v1:
   * if valid and fresh/stale:
     * return `Hit { entry: v1, source: V1, migrated: false, quarantined: flags }`
   * if corrupt:
     * `quarantine(v1_path, "corrupt")`
     * `flags.corrupt = true`
   * if unsupported:
     * `quarantine(v1_path, "unsupported_vN")`
     * `flags.unsupported = true`
   * if miss:
     * (continue)
4. Try legacy:
   * if valid legacy payload found:
     * `migrate_legacy_to_v1(key, payload, legacy_meta?)`
     * return `Hit { entry: migrated, source: Legacy, migrated: true, quarantined: flags }`
   * else:
     * (continue)
5. return `Miss { quarantined: flags }`

### 12.3 OsvClient::get_details (FetchOutcome as the final truth)

**Contract:**
* `OsvClient` owns online/offline semantics and network operations.
* `DetailsStore` owns storage law and migration.

**Algorithm:**
1. `store_result = details_store.load(key)`
2. If `store_result` is hit:
   * Map to `FetchOutcome::HitFresh` / `HitStale` / `HitLegacyMigrated`
   * Return immediately (no network)
3. If `store_result` is `Miss`:
   * If offline:
     * return `FetchOutcome::Error(OfflineCacheMiss)`
   * If online:
     * Execute fetch (retry, budgets, concurrency gate)
     * On success:
       * `details_store.save_v1_envelope(key, envelope{ source="fetch" ... })`
       * return `FetchOutcome::Fetched`
     * On failure:
       * return `FetchOutcome::Error(NetworkFailureClassified)`

---

## 13. Tests (must encode the law)

`tests/cache_contract.rs` MUST include:

### 13.1 Schema validation
* valid envelope decodes
* corrupt JSON quarantines
* unsupported `schema_version` quarantines and proceeds as miss

### 13.2 Legacy migration
* legacy valid → load returns hit and creates v1
* mapped timestamps obey §9.2 (Rule A)

### 13.3 Corruption & offline matrix
* Online: v1 corrupt → quarantine → fetch → v1 replaced → outcome fetched
* Offline: v1 corrupt + legacy valid → quarantine → legacy hit → v1 created → no error
* Offline: v1 corrupt + legacy miss → quarantine → error (only here)

### 13.4 Directory conflict
* create file at `.../osv/vulns/v1` → store access quarantines it and creates directory

---

## 14. Compatibility Policy

* v0.12 MUST support reading legacy as fallback until explicitly removed.
* Legacy removal MUST be gated by conditions (not a calendar date).

---

## 15. Security & Safety

* Cache files are untrusted input. Validate strictly.
* Quarantine preserves evidence and prevents silent data loss.
* Observability MUST avoid leaking sensitive payload fragments.
