# CLI reference

The authoritative parser is [src/cli.rs](../../src/cli.rs). Use `ghost --help` or `ghost <command> --help` for the full option list. Run Ghost as your desktop user; it refuses to run as root and calls `sudo` for steps that need it.

| Command | Behavior |
|---|---|
| `ghost install` | Installs missing packages and deploys configuration from the checkout. Re-run it to update |
| `ghost restore` | Undoes the most recent install; repeat to step further back |
| `ghost doctor` | Read-only checks; exits unsuccessfully if any check fails |
| `ghost apply` | Not implemented; fails |

## install

Run from the repository root, or pass `--source <DIR>`.

| Option | Effect |
|---|---|
| `--dry-run` | Print the plan; change nothing |
| `-y`, `--yes` | Skip Ghost's confirmation and pass `--noconfirm` to pacman |
| `--with <COMPONENT>` | Add an opt-in component (repeatable) |
| `--without <COMPONENT>` | Skip a default component (repeatable) |
| `--theme`, `--host`, `--translucent`, `--opacity` | Accepted by the parser but not implemented; using them is an error |

Components are defined in [ghost.toml](../../ghost.toml):

| Component | Default | Contents |
|---|---|---|
| `core` | Required | Hyprland, idle, lock, wallpaper, audio, clipboard, screenshot packages, `~/.config/hypr`, and the split-monitor-workspaces plugin |
| `shell` | Yes | Waybar, rofi, swaync, dock (style, launcher icon, pinned apps), launcher entry overrides |
| `apps` | Yes | Ghostty, Dolphin, Ark, Yazi with preview helpers; Tokyo Night GTK 3 theme, GTK 4 colours, KDE colour scheme, Ghostty and Yazi config; Tela blue icons |
| `sddm` | Opt-in | [Login theme](../guides/login-screen.md) and the drop-in that selects it |
| `gpu` | Opt-in | udev rule creating `/dev/dri/nvidia-dgpu`, then a udev reload |

A re-run keeps components installed earlier unless you pass `--without`.

Order of operations:

1. Show the components, missing packages, every file action, sources to fetch, and any follow-up commands.
2. If packages are missing and `pacman -Qu` reports pending updates, stop. Run `sudo pacman -Syu` first. Ghost never upgrades the system itself, so it can't cause a partial upgrade.
3. Ask for confirmation.
4. `sudo pacman -S --needed` the missing packages.
5. Write files, backing up anything replaced or removed.
6. Run the component hooks with sudo.
7. Fetch pinned sources, as you.

### Sources

Third-party code that isn't packaged for Arch is fetched from git at a full commit id named in `ghost.toml`, never a branch or tag. Ghost runs `git fetch --depth 1 <url> <commit>` and checks that the checkout is at that commit before using it. Bumping a pin means reviewing the upstream code at the new commit first.

| Source | Component | Installed as |
|---|---|---|
| [split-monitor-workspaces](https://github.com/Duckonaut/split-monitor-workspaces) | `core` | A checkout at `~/.config/hypr/plugins/split-monitor-workspaces` |
| [Tela-icon-theme](https://github.com/vinceliuice/Tela-icon-theme) | `apps` | Upstream's `install.sh -d ~/.local/share/icons blue`, run from a checkout in `$XDG_CACHE_HOME/ghost/sources/` |

A source is fetched when its pin changes, or when a checkout destination is no longer at the pinned commit. Ghost refuses to touch a destination that exists but isn't a git checkout. Sources are not backed up or removed by `restore`; delete the checkout or icon directories yourself if you no longer want them.

Each file is compared with what Ghost last deployed there:

| File state | Action |
|---|---|
| Missing | Created |
| Unchanged since Ghost wrote it | Updated |
| Exists, never deployed by Ghost | Backed up, then replaced |
| You edited it, Ghost's version changed | Left alone; new version written as `<file>.ghost-new` |
| You edited it, Ghost's version unchanged | Left alone |
| You deleted it | Not recreated |
| Symlink | Never replaced; new version written as `<file>.ghost-new` |
| No longer shipped, unedited | Backed up, then removed |
| No longer shipped, edited | Left in place, no longer tracked |

Files outside your home directory (`sddm`, `gpu`) are written with `sudo install` and `mv`. Paths in a user component must be under `~/`, and paths in a root component must be absolute. Every sudo write, move, or removal must fall inside a root component's destination in `ghost.toml` (directories may also be created on the way to one), so a tampered state file or backup index cannot direct sudo elsewhere. Installed files never keep group or other write permission, and `~/.local/state/ghost` and its backups are private (0700).

## restore

| Option | Effect |
|---|---|
| `--list` | List restorable backups |
| `--dry-run` | Show what would be put back or removed |
| `-y`, `--yes` | Skip confirmation |

`restore` puts every backed-up original back and removes files that install created. Files you changed since the install are kept. Ghost's record of what it deployed returns to its previous state, and the hooks of affected system components run again. Packages are never removed.

## doctor

Checks:
- **System:** Arch Linux, and whether the running kernel's modules are still installed (a pending reboot).
- **Packages:** the packages for the deployed components, and the Hyprland login session.
- **GPU and NVIDIA:** cards and drivers; open or proprietary module; `nvidia_drm` modeset and fbdev; video memory preservation across suspend; the multi-GPU device link.
- **Config:** the deployed Hyprland config, host profile, wallpapers, and lock screen image.
- **Session:** inside Hyprland only, `hyprctl configerrors`.

On the open modules with `UseKernelSuspendNotifiers=1` (the nvidia-utils default since 595), the nvidia-suspend services are not needed.

## Files

| Path | Contents |
|---|---|
| `$XDG_STATE_HOME/ghost/state.json` (default `~/.local/state/ghost/`) | Deployed components, file hashes, Ghost's git revision, installed source commits |
| `$XDG_STATE_HOME/ghost/backups/<id>/` | Originals and an index for one install; renamed `<id>.restored` once restored |
| `$XDG_CACHE_HOME/ghost/sources/` (default `~/.cache/ghost/sources/`) | Checkouts of sources installed by a `run` command |
