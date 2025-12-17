# OSV Details & Caching Strategy

Veil Guardian caches OSV vulnerability details (retrieved from `GET /vulns/{id}`) to enable smart offline capabilities and reduce API load.

## Why Raw JSON?

We store the OSV details as **raw JSON** (`serde_json::Value`) instead of a strongly-typed Rust struct.

### Reasons:
1.  **Schema Evolution**: The OSV schema is evolving. New fields may be added, or ecosystem-specific extensions might appear.
2.  **Resilience**: We want to avoid breaking the cache or the build if the upstream schema changes in a way that doesn't strictly adhere to our struct definition.
3.  **Forward Compatibility**: By storing raw JSON, we preserve all data returned by the API. We can extract new fields in the future without invalidating existing caches.

## Best-Effort Extraction

While the storage is raw, the `guardian` CLI performs a **best-effort extraction** of key fields for display:
- `summary`
- `severity` (CVSS)
- `affected` (versions/ranges)
- `references`

If a field is missing or malformed, it is simply omitted from the output rather than causing a crash or error.

## Cache Resilience & Layout (v0.11.x)

To ensure stability in high-concurrency environments (CI/CD) and against system crashes, the cache layer implements strict robustness guarantees:

1.  **Atomic Writes**: JSON files are written to a temporary file, synced to disk, and atomically renamed. This prevents partial writes during power failures.
2.  **File Locking**: Exclusive locks (`.lock` files) are used for all read/write operations, allowing safe parallel execution across multiple processes.
3.  **Versioning**:
    - **v1 Layout**: New cache entries are stored in a `v1` subdirectory with normalized filenames (e.g., `GHSA-foo-bar.json`).
    - **Collision Avoidance**: Keys containing unsafe characters are hashed to prevent filename collisions.
    - **Legacy Fallback**: The system transparently falls back to reading legacy cache paths if a v1 entry is missing, ensuring backward compatibility.
