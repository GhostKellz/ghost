<p align="center">
  <img src="assets/logo.png" width="240" alt="Ghost logo">
</p>
<h1 align="center">ghost</h1>
<p align="center">A Tokyo Night desktop for Arch Linux and Hyprland.</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-B7410E?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Lua-2C2D72?style=for-the-badge&logo=lua&logoColor=white" alt="Lua">
  <img src="https://img.shields.io/badge/Arch_Linux-1793D1?style=for-the-badge&logo=archlinux&logoColor=white" alt="Arch Linux">
  <img src="https://img.shields.io/badge/Hyprland-58E1FF?style=for-the-badge&logo=hyprland&logoColor=white" alt="Hyprland">
  <img src="https://img.shields.io/badge/Wayland-FFBC00?style=for-the-badge&logo=wayland&logoColor=white" alt="Wayland">
  <img src="https://img.shields.io/badge/NVIDIA-76B900?style=for-the-badge&logo=nvidia&logoColor=white" alt="NVIDIA">
</p>

---

**In development:** desktop configuration, theme assets, and an SDDM login theme are present. The Rust CLI installs, updates, and restores the desktop (`install`, `restore`) and diagnoses it (`doctor`); `apply` is not implemented yet. A live VM session still needs validation.

## Overview

Ghost brings a consistent Tokyo Night appearance to Hyprland, Waybar, rofi, notifications, the dock, and the lock screen. It uses modular Lua configuration with per-host monitor and GPU settings. The supplied workstation profile targets a dual-monitor NVIDIA desktop; unknown hosts use automatic output configuration.

## Features

- **Floating dock:** frosted glass, rounded corners, blue hover glow, auto-hide at the bottom edge, and a rofi launcher button.
- **Keyboard workflow:** master/dwindle toggle, Vim-style window controls, per-monitor workspaces, clipboard history, annotated screenshots, and a SUPER+/ keybind help list.
- **Desktop shell:** per-output Waybar with a workspace pager, swaync notifications, PipeWire controls, and Dolphin integration.
- **Themed applications:** Tokyo Night for GTK 3, GTK 4 and KDE/Qt apps, macOS-style title buttons, and a translucent Ghostty.
- **Login screen:** an SDDM theme with a frosted-glass panel.
- **Display profiles:** workstation and VM examples, with a generic fallback for other hosts.
- **OLED-conscious idle settings:** blank displays after ten minutes and lock after fifteen; no automatic suspend.
- **Theme assets:** Night and Storm palettes. The checked-in application colors use Night; automatic theme switching is planned. Supply your own desktop and lock-screen images.

## Quick start

With a Rust toolchain that supports the edition in [Cargo.toml](Cargo.toml):

```bash
git clone https://github.com/GhostKellz/ghost.git
cd ghost
cargo build --locked --release
./target/release/ghost doctor
./target/release/ghost install --dry-run
```

`ghost install` shows its plan and asks before installing packages or writing files; `ghost restore` undoes it. Try it on a test machine or VM first; see [installation](docs/getting-started/installation.md).

## Configuration

| Change | Source |
|---|---|
| Displays and GPU selection | `config/hypr/hosts/` |
| Layout, gaps, borders, blur | `config/hypr/ghost/look.lua` |
| Keyboard and mouse bindings | `config/hypr/ghost/binds.lua` |
| Startup programs and dock arguments | `config/hypr/ghost/autostart.lua` |
| Dock opacity and hover styling | `config/nwg-dock-hyprland/style.css` |
| Theme colors | `themes/*/palette.toml` and application color files |
| Packages and deployed files | `ghost.toml` |

See [configuration](docs/getting-started/configuration.md), [themes and dock](docs/guides/themes-and-dock.md), and the [keybindings](docs/reference/keybindings.md).

## Project structure

```text
ghost.toml    Components: packages and where each file is deployed
src/          Rust CLI: install, restore, doctor
config/       Hyprland and desktop application configuration
system/       SDDM login theme and the NVIDIA udev rule
themes/       Tokyo Night palette definitions
docs/         Setup, guides, reference, and architecture
```

`ghost install` copies configuration into place. The desktop then loads those files directly; nothing is generated yet. See the [architecture](docs/internals/architecture.md).

## Documentation

Start with the [documentation index](docs/README.md). The [roadmap](docs/development/roadmap.md) separates implemented files from planned work.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for formatting, tests, and Conventional Commits. Changes are recorded in [CHANGELOG.md](CHANGELOG.md).

## Security

Report vulnerabilities privately using [SECURITY.md](SECURITY.md). Dependency decisions live in [advisories](docs/advisories/triage.md).

## License

Project code is [MIT licensed](LICENSE). Palette comments identify their upstream source.
