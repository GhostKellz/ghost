# Manual VM installation

The installer is not implemented. This procedure is for a dedicated Arch test account in a VM, with a recoverable console and a snapshot taken before package or configuration changes. It does not change your login manager.

## Prerequisites

Use an updated Arch installation with working graphics drivers, a usable login session, and a Hyprland build supporting the Lua API used in this repository. Match its installed version to the [upstream versioned documentation](https://wiki.hypr.land/version-selector/). Record `Hyprland --version` during validation.

On a passthrough VM, confirm which outputs the guest exposes and whether the passed-through GPU has a monitor or dummy plug. A virtual console alone does not demonstrate rendering on the NVIDIA GPU. Verify the renderer and output mapping in the guest rather than inferring acceleration from the display connection.

Review [packages/hyprland.txt](../../packages/hyprland.txt). It describes desktop packages, not a complete base OS or GPU driver installation. Several dock pins reference optional applications outside that manifest; remove unavailable pins or install those applications separately.

From the cloned repository, in **Bash**, install the reviewed manifest using a full system upgrade:

```bash
mapfile -t ghost_packages < <(awk 'NF && $1 !~ /^#/ {print $1}' packages/hyprland.txt)
sudo pacman -Syu --needed "${ghost_packages[@]}"
```

Check package availability and any provider/conflict prompts on the guest. Do not substitute similarly named packages without checking their CLI and configuration compatibility.

## Copy into a clean test account

The following Bash block refuses to replace existing configuration directories or Ghost data. For an existing desktop, back up those paths outside the repository and use a separate test account first. Run from the repository root:

```bash
set -e
for component in hypr waybar rofi swaync nwg-dock-hyprland; do
    if [[ -e "$HOME/.config/$component" || -L "$HOME/.config/$component" ]]; then
        echo "Existing config: $component; use a clean test account." >&2
        exit 1
    fi
done
if [[ -e "$HOME/.local/share/ghost" || -L "$HOME/.local/share/ghost" ]]; then
    echo "Existing Ghost data; use a clean test account." >&2
    exit 1
fi
mkdir -p "$HOME/.config" "$HOME/.local/share/ghost/wallpapers"
for component in hypr waybar rofi swaync nwg-dock-hyprland; do
    cp -a "config/$component" "$HOME/.config/"
done
cp wallpapers/tokyonight/night/arch_*.png "$HOME/.local/share/ghost/wallpapers/"
cp wallpapers/tokyonight/night/stripes_*.png "$HOME/.local/share/ghost/wallpapers/"
cp wallpapers/tokyonight/night/lockscreen_00_3840x2160.png "$HOME/.local/share/ghost/lockscreen.png"
```

The wallpaper files are copied directly into the configured directory, avoiding any assumption about recursive scanning. The lock screen gets its own image. Personal wallpaper collections are not part of this setup.

Review [host selection](configuration.md) before launch, especially if the hostname is `arch`. For a generic VM, use automatic outputs first. This procedure intentionally does not install the workstation-specific udev rule or an optional workspace package.

## Launch and validate

From the guest's local TTY, as the test user, start the compositor using the installed package's supported launcher. For installations supporting direct launch:

```bash
Hyprland
```

Keep a separate recovery console available. In the new session run `hyprctl configerrors`, `hyprctl monitors`, and `hyprctl layers`, then work through [the acceptance checklist](../guides/troubleshooting.md). A clean config-error list does not validate shell applications, lock behavior, GPU acceleration, or screenshots.

## Rollback

End the test session from your recovery console or login manager. Restore the VM snapshot to undo the entire trial, including packages. To undo only a clean-account config deployment, remove only the five directories and Ghost data created by the copy step after confirming they contain no later work. If testing against backed-up configuration, restore each original path after ending the session. `ghost restore` cannot perform this yet.
