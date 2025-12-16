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
