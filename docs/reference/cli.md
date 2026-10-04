# CLI reference

The authoritative parser is [src/cli.rs](../../src/cli.rs). Use `cargo run --locked -- --help` or `<command> --help` to inspect it.

| Command | Accepted options | Current behavior |
|---|---|---|
| `ghost install` | `--dry-run`, `--theme <STRING>`, `--host <STRING>`, `--translucent`, `--opacity <50..100>` | Fails; no package or file actions |
| `ghost apply` | `--theme <STRING>` | Fails; no theme rendering |
| `ghost restore` | None | Fails; no backup restoration |
| `ghost doctor` | None | Fails; no diagnostics |

Every operational command returns an error naming the unimplemented subcommand and exits unsuccessfully. Help and version requests are handled by clap. Missing subcommands, unknown options, and invalid opacity values are parser errors.

The help text describes the intended future contract. In particular, `--dry-run` does not yet produce a plan, theme and host strings are not validated against repository files, and `--opacity` does not yet enable or render translucency. Do not automate deployment around these flags until the relevant implementation and behavior tests exist.
