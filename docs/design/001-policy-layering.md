# v0.9.x Design: Policy Layering & Configuration

**Status**: Draft
**Target Version**: v0.9.0
**Epic**: P (Policy)

## Objective
Define a robust configuration precedence model to support individual developers, repositories, and organizations simultaneously. This ensures Veil can scale from personal use to enterprise enforcement.

## Layering Model (Precedence)

Veil will resolve configuration in the following order (Highest to Lowest priority):

1.  **CLI Arguments** (`--fail-on-severity`, `--ignore-path`, etc.)
    *   *Rationale*: explicit user intent for a specific run should always win.
2.  **Organization Policy** (Proposed)
    *   *Source*: `VEIL_ORG_CONFIG` env var (URL or Path), or system global `/etc/veil/org.toml`.
    *   *Constraint*: Can set "Minimum Standards" (e.g., `min_fail_on_severity`). If Repo config is looser than Org, Org wins.
3.  **Repository Config** (`./veil.toml`)
    *   *Source*: Project root.
    *   *Purpose*: Project-specific overrides, ignores, and rule selections.
4.  **User Global Config** (`$XDG_CONFIG_HOME/veil/veil.toml` or `~/.config/veil/veil.toml`)
    *   *Purpose*: User preferences (e.g., default output format, global ignores for tools/IDEs).
5.  **Built-in Defaults**
    *   *Purpose*: Safe fallbacks (App profile default).

## Merger Logic

### 1. `Option<T>` (Scalar values)
Usually "Last One Wins" (Higher priority replaces lower).
*   Exception: **Safety Floors**.
    *   If Org sets `fail_on_severity = "High"`, Repo cannot set it to "Critical" (looser).
    *   Logic: `effective_policy = max(org_policy, repo_policy)` where `max` means "Most Strict".

### 2. `Vec<T>` / `HashSet<T>` (Collections like `ignore`)
*   **Merge Strategy**: Union.
    *   `effective_ignore = user_ignore + repo_ignore + org_ignore`.
    *   Rationale: If *any* layer says "ignore this", it's usually safe or necessary to ignore.
*   **Rules Allowlist**: Intersection (maybe?).
    *   *TBD*: If Org says "Only allow these rules", Repo cannot add more? Or Org says "Must check these"?
    *   *Proposal*: `rules` config is additive. Remote rules (from Org) are added to local rules.

## Implementation Plan

### v0.9.0
*   Define the `ConfigLoader` trait/struct structure to support multiple sources.
*   Impl `load_effective_config()` that performs the layered merge.
*   Add `veil config dump` command to inspect the final merged configuration (debug tool).

### Future
*   Remote config fetching (HTTP for Org Policy).
