# Changelog

All notable changes to ghost are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project scaffold.
- `ghost install`: installs missing packages and deploys configuration per component from `ghost.toml`. Includes a dry run, a confirmation prompt, a three-way update that writes `.ghost-new` next to files you edited, backups, and a stop when system updates are pending.
- `ghost restore`: undoes the latest install, stepping back through backups.
- `ghost doctor`: read-only checks for packages, NVIDIA driver and suspend setup, multi-GPU device link, pending reboot, deployed config, and Hyprland config errors.
- SDDM login theme (`--with sddm`): frosted-glass panel over the wallpaper or a built-in blue glow, power controls with confirmation, session selector. The udev rule component (`--with gpu`).
- Tokyo Night for applications: `Ghost-TokyoNight` GTK 3 theme with macOS-style title buttons, GTK 4 colours, a KDE colour scheme for Dolphin via the KDE platform theme, and Ghostty on frosted glass.
- Waybar workspace pager; SUPER+/ keybind help generated from bind descriptions; SUPER+Q closes windows.
- Launcher overrides hiding developer and system tools; Ghostty keeps its own icon.
- Yazi terminal file manager (Super+Shift+E) alongside Dolphin, themed to match Ghostty.
- Optional host profile `startup` commands; the `arch-dev` profile renders on its passed-through GPU to a headless output for VNC.

### Changed
- Dock auto-hides and reveals at the bottom edge, with a blue launcher button and no per-icon backgrounds; Waybar and the dock are more translucent.

### Fixed
- Dock pins were installed to a path nwg-dock never reads; they now go to `~/.cache/nwg-dock-pinned`.

### Removed
- `packages/hyprland.txt`; package lists now live in `ghost.toml`.

<!--
On release, copy the [Unreleased] entries under a new dated heading, e.g.:

  ## [1.0.0] - 2026-07-05

Then reset [Unreleased] to empty groups and add a link reference at the bottom:

  [1.0.0]: https://github.com/GhostKellz/ghost/releases/tag/v1.0.0

Change groups (in this order; omit any that are empty):
  Added       new features
  Changed     changes in existing functionality
  Deprecated  soon-to-be removed features
  Removed     now-removed features
  Fixed       bug fixes
  Security    vulnerability fixes

Semantic Versioning - given MAJOR.MINOR.PATCH, increment the:
  MAJOR  for incompatible API changes
  MINOR  for backward-compatible new functionality
  PATCH  for backward-compatible bug fixes
-->

[Unreleased]: https://github.com/GhostKellz/ghost/commits/main
