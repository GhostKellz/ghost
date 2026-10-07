# Security Policy

## Reporting a Vulnerability

**Please do not open public GitHub issues for security vulnerabilities.**

Report privately through GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Report a vulnerability" under the repository's **Security** tab), or by
contacting the maintainers directly.

Please include:

- A description of the vulnerability and its impact
- Steps to reproduce or a proof of concept
- Affected component(s) and version/commit
- Any suggested remediation, if known

### Response Targets

| Stage | Target |
|-------|--------|
| Acknowledgement | within 72 hours |
| Initial assessment | within 7 days |
| Fix / mitigation | depends on severity |
| Public disclosure | coordinated, after a fix is available |

These are good-faith targets, not contractual SLAs.

## Supported Versions

ghost is pre-1.0. Only the latest commit on the default branch receives
security fixes.

| Version | Supported |
|---------|-----------|
| `main` (latest) | ✅ |
| older tags | ❌ |

## Disclosure Policy

We follow coordinated disclosure. Please allow a reasonable window to ship a fix
before public discussion. Reporters who wish to be credited will be acknowledged.

## Privilege Boundaries

| Component | Privilege / data access |
|-----------|-------------------------|
| Rust CLI | User process; refuses to run as root. Calls `sudo` for `pacman -S --needed`, root component files, and their hooks; each step is listed in the plan first. Sudo file operations are confined to root components' destinations in `ghost.toml`. Pinned git sources run as the user after `sudo -k` |
| Desktop configuration | Runs commands as the logged-in user |
| Clipboard history | Stores copied content through cliphist; may include sensitive text |
| Backups and state | `~/.local/state/ghost/` (0700); holds copies of replaced user and system files |
| SDDM theme | Runs inside the greeter as the `sddm` user; passes credentials only to SDDM's `login` call |
| Optional udev rule | System configuration; installed only with `ghost install --with gpu` |

Idle display blanking does not lock the session immediately. The supplied policy
blanks after ten minutes and locks after fifteen. Test hypridle and hyprlock in
the target session before relying on either explicit or automatic locking.

## Dependency Review

Run `cargo audit` with a current advisory database when available. Record actual
results and decisions in [advisories](docs/advisories/triage.md); unit tests do
not constitute an advisory scan. Review logs and window titles for private data
before including diagnostics in public issues.
