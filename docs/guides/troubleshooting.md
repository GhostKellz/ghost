# Troubleshooting and VM acceptance

Collect diagnostics from inside the test session:

```bash
Hyprland --version
hyprctl configerrors
hyprctl monitors
hyprctl clients
hyprctl layers
hyprctl binds
systemctl --user status hyprpolkitagent
journalctl --user -b -p warning
```

Review output before sharing: client titles and logs can contain private information. Record the repository revision and the guest package versions alongside a screenshot.

## Common failures

| Symptom | Check |
|---|---|
| No session / wrong GPU | Guest driver state, compositor logs, and any `AQ_DRM_DEVICES` path in the selected host profile |
| Unexpected automatic monitor layout | Exact `/etc/hostname` (no matching profile falls back to `hosts/default.lua` silently); a broken profile shows an on-screen `ghost:` error with the original message |
| A `ghost:` error notification at login | The named step failed and was skipped; later steps still ran. Details are in the notification, the Hyprland log (`[Lua] ghost:`), and `hyprctl configerrors` |
| Dock not on DP-3 | Only `arch` names DP-3; validate output names before editing `monitor_priority` |
| Dock has missing pins | Confirm matching `.desktop` files exist; the package manifest does not install every pinned app |
| Flat dock / no blur | Dock namespace from `hyprctl layers`, layer rule matching, surface opacity, and blur settings |
| Wallpaper or lock image missing | Files at the separate wallpaper and lock-screen paths; do not assume recursive scanning |
| Wrong app placement | Actual case-sensitive class/title from `hyprctl clients` versus `ghost/rules.lua` |
| Locks do nothing | hypridle process, hyprlock launch errors, and session association in logind |
| Network controls do nothing | NetworkManager service availability; installing an editor alone does not configure networking |
| Dolphin “Open with” is empty | `archlinux-xdg-menu`, `XDG_MENU_PREFIX=arch-`, then run `kbuildsycoca6` in the session |
| Qt apps do not match | qt6ct/Kvantum packages are supplied, but theme selection/configuration is not bundled |

## Acceptance checklist

- [ ] No compositor configuration errors; record installed version and selected profile.
- [ ] Outputs, resolution, refresh, scaling, and renderer match the guest hardware.
- [ ] Terminal, launcher, file manager, screenshot picker, clipboard, and notifications work.
- [ ] Dock autohides, launches rofi, floats above the edge, and shows the intended blur/hover effect.
- [ ] Test focus, move, resize, maximize, layout switching, and workspace selection on each output.
- [ ] Test optional workspace integration separately from native fallback behavior.
- [ ] Confirm Waybar controls, audio, network editor, tray, and polkit prompts.
- [ ] Test explicit lock, password unlock, and lock-plus-blank with a recovery console available.
- [ ] Check the idle blank and later lock sequence; blanking alone leaves a five-minute unlocked interval by design.
- [ ] Verify wallpaper cycling and the independent lock-screen image.
- [ ] Log out and back in; check that each startup program has the expected number of processes.

Complete this checklist in the target session before treating the desktop configuration as validated.
