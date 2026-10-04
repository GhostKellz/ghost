# Configuration

The entry point is [hyprland.lua](../../config/hypr/hyprland.lua). Modules under `ghost/` define environment, monitors, appearance, input, workspaces, bindings, rules, and startup programs.

## Host selection

`ghost/host.lua` reads `/etc/hostname`, trims surrounding whitespace, and looks for `hosts/<hostname>.lua`. If no such file exists, `hosts/default.lua` is used without a message. If the file exists but fails to load — a Lua error, or a return value that is not a table with a `monitors` list — Ghost shows a Hyprland error notification quoting the original error and then uses `hosts/default.lua`. Every profile must return a table containing `monitors` (an empty list is valid).

| Profile | Intended environment | Assumptions |
|---|---|---|
| `arch` | Workstation | DP-3 left, DP-2 right; both 4K at 240 Hz, scale 1; HDR requested; NVIDIA device symlink required |
| `arch-dev` | Test VM | Automatic monitor rules; NVIDIA environment values; no fixed output or primary dock monitor |
| `default` | Unknown hostname | Preferred mode, automatic position and scale; no GPU overrides |

Create a profile matching the test machine's hostname after inspecting `hyprctl monitors`. Copy the default profile as a starting point and add only settings you have verified. The CLI's `--host` option is not implemented and cannot override runtime selection.

The `arch` profile sets `AQ_DRM_DEVICES=/dev/dri/nvidia-dgpu`. Its matching [udev rule](../../system/udev/61-ghost-nvidia-dgpu.rules) assumes one NVIDIA GPU and is not installed automatically. Confirm the device path before using this profile. Do not copy the workstation's display modes, HDR settings, or GPU selection onto an unrelated machine.

## Runtime paths

The manual setup uses standard `~/.config/` paths. The workspace module honors `XDG_CONFIG_HOME` when searching for its optional Lua package, but the supplied lock screen and wallpaper paths remain under `~/.local/share/ghost/`.

| Files | Purpose |
|---|---|
| `hypr/ghost/report.lua` | On-screen error and warning notifications for config problems |
| `hypr/ghost/look.lua`, `input.lua` | Layout, borders, blur, keyboard and pointer settings |
| `hypr/ghost/workspaces.lua` | Five numbered workspace shortcuts, optional split-monitor integration |
| `hypr/ghost/rules.lua` | Window matching and five layer blur rules |
| `hypr/ghost/autostart.lua` | Shell programs, clipboard watcher, and polkit agent |
| `hypr/hypridle.conf`, `hyprlock.conf` | Idle actions and lock screen |
| `hypr/hyprpaper.conf` | Wallpaper directory, random order, half-hour interval |
| `waybar/`, `rofi/`, `swaync/`, `nwg-dock-hyprland/` | Application UI configuration |

## Workspaces and startup

The optional `split-monitor-workspaces` package is searched for under `~/.config/hypr/plugins/split-monitor-workspaces/lua/`. It is not bundled or installed by the current CLI. Without it, Ghost shows a warning notification and uses native `m~N` and `m±1` selectors; this does not pre-create five workspaces per output. If the package is present but fails to load, an error notification shows the cause. Verify workspace behavior separately with and without the package.

Startup launches Waybar, swaync, hypridle, hyprpaper, the dock, a clipboard watcher, and the user polkit service. Starting a fresh session exercises this hook; a config reload should not be treated as a restart of these programs.

If adopting uwsm later, move relevant environment settings to its session environment files as described in the [upstream environment guide](https://wiki.hypr.land/Configuring/Advanced-and-Cool/Environment-variables/). The current manual test guide does not configure uwsm.
