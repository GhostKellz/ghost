# Development roadmap

## Present in the working tree

- Rust CLI: `install` (packages, three-way file deployment, backups), `restore`, and `doctor`. `apply` still fails explicitly.
- Component manifest (`ghost.toml`) with packages and file mappings.
- Modular Hyprland Lua config with host profiles and optional workspace integration.
- Waybar, rofi, swaync, dock, idle, lock-screen, and wallpaper configuration.
- SDDM login theme.
- Night and Storm palette definitions.
- Workstation-specific udev rule, installed by the `gpu` component.

Presence of a configuration file does not establish runtime compatibility. Complete the [VM checklist](../guides/troubleshooting.md) in the target session.

## Next milestones

1. **Validate the desktop in the VM.** Confirm GPU/output routing, compositor APIs, app classes, shell styling, lock/idle behavior, and native workspace fallback. Record results before treating the configs as a working baseline.
2. **Check asset provenance before distribution.** Wallpaper and palette attribution must be confirmed before a release.
3. **Implement theme rendering.** Read palettes, validate roles, generate all supported surfaces consistently, and test opacity boundaries and Night/Storm switching.
4. **Finish installation.** Validate the package and root-file steps on the VM; migrations for config layout changes; host selection.
5. **Extend diagnostics and automation.** Monitor mode, bit depth, and VRR checks inside Hyprland; JSON output; CI for Rust and static config validation.

Session-manager integration, real desktop screenshots, and release packaging follow validated behavior. No release date is implied by this list.
