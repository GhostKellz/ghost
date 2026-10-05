# Configuration

The entry point is [hyprland.lua](../../config/hypr/hyprland.lua). Modules under `ghost/` define environment, monitors, appearance, input, workspaces, bindings, rules, and startup programs.

## Host selection

`ghost/host.lua` reads `/etc/hostname`, trims surrounding whitespace, and looks for `hosts/<hostname>.lua`. If no such file exists, `hosts/default.lua` is used without a message. If the file exists but fails to load — a Lua error, or a return value that is not a table with a `monitors` list — Ghost shows a Hyprland error notification quoting the original error and then uses `hosts/default.lua`. Every profile must return a table containing `monitors` (an empty list is valid).

| Profile | Intended environment | Assumptions |
|---|---|---|
| `arch` | Workstation | DP-3 left, DP-2 right; both 4K at 240 Hz, scale 1; HDR requested; NVIDIA device symlink required |
| `arch-dev` | Passthrough test VM | Renders only on the NVIDIA GPU (`AQ_DRM_DEVICES`) to a 1920×1080 headless output `VNC-1`, served by wayvnc on localhost; see below |
| `default` | Unknown hostname | Preferred mode, automatic position and scale; no GPU overrides |

A profile may also return `startup`, a list of shell commands run before the shell programs when the session starts.

Create a profile matching the test machine's hostname after inspecting `hyprctl monitors`. Copy the default profile as a starting point and add only settings you have verified. The CLI's `--host` option is not implemented and cannot override runtime selection.

The `arch` profile sets `AQ_DRM_DEVICES=/dev/dri/nvidia-dgpu`. Its matching [udev rule](../../system/udev/61-ghost-nvidia-dgpu.rules) assumes one NVIDIA GPU and is installed only with `ghost install --with gpu`. `ghost doctor` warns when the link is missing on a multi-GPU machine. Confirm the device path before using this profile. Do not copy the workstation's display modes, HDR settings, or GPU selection onto an unrelated machine.

## Runtime paths

The manual setup uses standard `~/.config/` paths. The workspace module honors `XDG_CONFIG_HOME` when searching for its optional Lua package, but the supplied lock screen and wallpaper paths remain under `~/.local/share/ghost/`.

| Files | Purpose |
|---|---|
| `hypr/ghost/report.lua` | On-screen error and warning notifications for config problems |
| `hypr/ghost/look.lua`, `input.lua` | Layout, borders, blur, keyboard and pointer settings |
| `hypr/ghost/workspaces.lua` | Five numbered workspace shortcuts, optional split-monitor integration |
| `hypr/ghost/rules.lua` | Window matching and five layer blur rules |
| `hypr/ghost/autostart.lua` | Profile `startup` commands, shell programs, clipboard watcher, polkit agent, GTK settings |
| `hypr/ghost/env.lua` | Cursor, Qt/KDE and GTK theme selection, profile environment |
| `hypr/scripts/keybinds.sh` | SUPER+/ keybind help |
| `hypr/hypridle.conf`, `hyprlock.conf` | Idle actions and lock screen |
| `hypr/hyprpaper.conf` | Wallpaper directory, random order, half-hour interval |
| `waybar/`, `rofi/`, `swaync/`, `nwg-dock-hyprland/` | Application UI configuration |

## Workspaces and startup

The optional `split-monitor-workspaces` package gives each monitor its own five persistent workspaces. Ghost loads it from `~/.config/hypr/plugins/split-monitor-workspaces/lua/`; the CLI does not install it yet. Clone the tag matching `Hyprland --version` (pure Lua, no build step):

```bash
git clone --depth 1 --branch v0.56.2 https://github.com/Duckonaut/split-monitor-workspaces ~/.config/hypr/plugins/split-monitor-workspaces
```

Without it, Ghost shows a warning notification and uses native `m~N` and `m±1` selectors; this does not pre-create five workspaces per output. If the package is present but fails to load, an error notification shows the cause. Verify workspace behavior separately with and without the package.

Startup runs the profile's `startup` commands, then launches Waybar, swaync, hypridle, hyprpaper, the dock, a clipboard watcher, and the user polkit service, and sets the GTK dark-mode, theme, icon, and button-layout settings. Starting a fresh session exercises this hook; a config reload should not be treated as a restart of these programs.

If adopting uwsm later, move relevant environment settings to its session environment files as described in the [upstream environment guide](https://wiki.hypr.land/Configuring/Advanced-and-Cool/Environment-variables/). The current manual test guide does not configure uwsm.

## Passthrough VM without a monitor

With a virtual console GPU plus a passed-through NVIDIA GPU that has no display attached, Hyprland would pick the virtual device as primary, render on the NVIDIA GPU, and stall copying frames between them. The `arch-dev` profile avoids this by limiting Hyprland to the NVIDIA GPU (the `gpu` component's `/dev/dri/nvidia-dgpu` link) and creating a headless output, `VNC-1`, which `wayvnc` serves on `127.0.0.1:5900`. The virtual console then shows only the login screen and TTYs.

To view the desktop, tunnel the port over SSH and connect a VNC client to `localhost`; most clients, including KRDC, can open the tunnel themselves. VNC is CPU-encoded and suited to checking appearance, not judging smoothness. `wayvnc` is test tooling on that VM, not a Ghost package.
