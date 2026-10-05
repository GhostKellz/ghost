# Installation

Ghost is still being validated. Try it on a test machine or VM first, with a recoverable console, and take a snapshot before installing. Packages and system files change; your login manager changes only if you choose the `sddm` component.

## Prerequisites

Use an up-to-date Arch installation with working graphics drivers and a Hyprland build that supports the Lua API this repository uses. Match it to the [upstream versioned documentation](https://wiki.hypr.land/version-selector/) and record `Hyprland --version` during validation. Ghost doesn't install GPU drivers; `ghost doctor` reports driver problems and suggests fixes.

On a passthrough VM, confirm which outputs the guest exposes and whether the passed-through GPU has a monitor or dummy plug. A virtual console alone does not demonstrate rendering on the NVIDIA GPU.

## Build and check

```bash
git clone https://github.com/GhostKellz/ghost.git
cd ghost
cargo build --locked --release
./target/release/ghost doctor
```

## Install

From the repository root, as your desktop user:

```bash
./target/release/ghost install --dry-run
./target/release/ghost install
```

The plan lists missing packages, every file change, and any follow-up commands, then asks before acting. Add opt-in components with `--with sddm` or `--with gpu`, and skip defaults with `--without`. If your system has pending updates, Ghost stops and asks you to run `sudo pacman -Syu` first. The [CLI reference](../reference/cli.md) covers each file case and option.

Files you have edited are never overwritten: Ghost writes its new version beside yours as `<file>.ghost-new`. Anything it replaces is backed up.

Ghost ships no wallpapers. Copy your own images into `~/.local/share/ghost/wallpapers/` (hyprpaper cycles them) and one image to `~/.local/share/ghost/lockscreen.png` (the lock screen). See [wallpapers](../guides/themes-and-dock.md#wallpapers).

Review [host selection](configuration.md) before launch, especially if the hostname is `arch`. The `arch` profile requires the `gpu` component.

## Launch and validate

From a TTY, or the Hyprland entry in your login manager, start the compositor:

```bash
Hyprland
```

Keep a separate recovery console available. Run `ghost doctor` inside the session; it adds `hyprctl configerrors` to its checks. Then run `hyprctl monitors` and `hyprctl layers`, and work through [the acceptance checklist](../guides/troubleshooting.md). A clean config-error list does not validate shell applications, lock behavior, GPU acceleration, or screenshots.

## Update

Pull the repository, rebuild, and run `ghost install` again. Components installed earlier stay installed. Review any `.ghost-new` files it reports and merge what you want.

## Roll back

`ghost restore` undoes the most recent install: originals come back, files Ghost created are removed unless you changed them, and system-file hooks run again. Repeat it to step further back; `ghost restore --list` shows what's available. Packages are not removed. Restore the VM snapshot to undo a whole trial, including packages.
