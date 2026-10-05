# Dependency inventory

| Surface | Source of truth | Review scope |
|---|---|---|
| Rust CLI | [Cargo.toml](../../Cargo.toml), [Cargo.lock](../../Cargo.lock) | Direct dependencies: clap, anyhow, serde, serde_json, toml, sha2; inspect the lockfile for transitive versions |
| Desktop packages | [ghost.toml](../../ghost.toml) | Per-component lists, all from Arch's official repositories; no AUR packages |
| Optional workspace integration | [workspaces.lua](../../config/hypr/ghost/workspaces.lua) | External Lua package, not bundled or fetched by the current CLI |
| Wallpapers and palettes | [theme guide](../guides/themes-and-dock.md) | Preserve licenses and verify provenance before release |

Use `cargo tree --locked` to inspect dependency relationships and `cargo audit` when the scanner and advisory database are available. A successful unit test run is not a dependency vulnerability scan. No scanner result is asserted by this inventory.

Dependabot groups minor and patch updates for Cargo and GitHub Actions. Major updates require manual review. There is no checked-in CI workflow yet.
