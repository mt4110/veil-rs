# Configuration Layering (v0.9.0)

Veil supports a layered configuration system to allow flexible policy management across organizations, users, and repositories.

## Precedence Order

Configuration is merged in the following order (later layers override earlier ones):

1. **User Layer** (Base preferences)
2. **Organization Layer** (Corporate policies / Defaults)
3. **Repository Layer** (Project-specific overrides)

**Effective Config** = `Default` + `User` + `Org` + `Repo`.

> Note: Future versions may introduce "Hard Policies" where Org config can enforce certain rules that cannot be overridden by Repo config. For v0.9.0, Org config acts as "Defaults" that override User preferences but can be overridden by the project.

## Layer Resolution

### 1. Repository Layer
- **Explicit**: `--config <PATH>` (Fail-fast)
- **Implicit**: `./veil.toml` (in current directory)

### 2. Organization Layer
- **Explicit**: `VEIL_ORG_CONFIG` (Fail-fast)
- **Implicit**:
    1. `$XDG_CONFIG_HOME/veil/org.toml` (or `~/.config/veil/org.toml`)
    2. `/etc/veil/org.toml`
    3. `VEIL_ORG_RULES` (Legacy support, implicitly deprecated)

### 3. User Layer
- **Explicit**: `VEIL_USER_CONFIG` (Fail-fast)
- **Implicit**: `$XDG_CONFIG_HOME/veil/veil.toml` (or `~/.config/veil/veil.toml`)

## Debugging

Use the `veil config dump` command to inspect specific layers or the merged result:

```bash
# View effective config (merged)
veil config dump

# View specific layer
veil config dump --layer org
veil config dump --layer user
veil config dump --layer repo
```
