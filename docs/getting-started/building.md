# Building and checks

Use a Rust toolchain supporting the edition declared in [Cargo.toml](../../Cargo.toml). Cargo.lock records the dependency resolution. No Rust build is needed to test the desktop configuration.

From the repository root:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --help
cargo run --locked -- install --help
```

The command tests cover clap's definition, rejection of an out-of-range opacity, and an unimplemented command error. They do not test installation or desktop behavior. All operational subcommands currently fail intentionally; see [CLI reference](../reference/cli.md).

## Configuration checks

With Lua installed, parse each module without running it:

```bash
find config/hypr -name '*.lua' -exec luac -p {} \;
```

This checks Lua syntax only. Plain Lua does not provide Hyprland's `hl` API; a successful parse does not verify options, dispatchers, or rules. Waybar's config is JSONC, and must be parsed with comment support. Palette TOML files should have every role resolve to a declared color.

Run [the runtime checklist](../guides/troubleshooting.md) in the target session before calling a desktop change validated. There is currently no CI workflow in this repository; Dependabot configuration alone does not run tests.
