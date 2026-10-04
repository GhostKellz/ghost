# Contributing to ghost

Thanks for your interest in contributing to ghost! This guide covers the
basics for getting changes merged.

## Getting Started

1. Fork the repository on GitHub.
2. Clone your fork and create a feature branch from `main`:

```bash
git clone https://github.com/GhostKellz/ghost
cd ghost
git checkout -b feature/your-change
```

## Development Workflow

- Keep changes focused; one logical change per pull request.
- Match the existing code style and run the project's formatter/linter.
- Add or update tests for any behavior change.
- Update documentation under `docs/` when relevant.

Run the current Rust checks from the repository root:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

For desktop changes, parse the Lua files and complete the relevant checks in
the [VM acceptance guide](docs/guides/troubleshooting.md). Include the tested
Hyprland version and distinguish syntax checks from a real session test.
Keep documentation linked from the single [docs index](docs/README.md).

## Commit Messages

Follow Conventional Commits:

```
type(scope): short description
```

Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`.

## Pull Requests

Before opening a PR:

- Rebase on the latest `main`.
- Ensure the build, tests, and linters pass.
- Describe **what** changed and **why**.

## Reporting Bugs

Open an [issue](https://github.com/GhostKellz/ghost/issues) with steps to
reproduce, expected vs. actual behavior, and environment details.

## License

By contributing, you agree your contributions are licensed under the MIT License.
