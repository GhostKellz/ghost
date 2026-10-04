# Development roadmap

## Present in the working tree

- Rust CLI parser, argument tests, and explicit failures for unfinished commands.
- Modular Hyprland Lua config with host profiles and optional workspace integration.
- Waybar, rofi, swaync, dock, idle, lock-screen, and wallpaper configuration.
- Night and Storm palette definitions.
- Arch package manifest and a workstation-specific udev rule.

Presence of a configuration file does not establish runtime compatibility. Complete the [VM checklist](../guides/troubleshooting.md) in the target session.

## Next milestones

1. **Validate the desktop in the VM.** Confirm GPU/output routing, compositor APIs, app classes, shell styling, lock/idle behavior, and native workspace fallback. Record results before treating the configs as a working baseline.
2. **Check asset provenance before distribution.** Wallpaper and palette attribution must be confirmed before a release.
3. **Implement theme rendering.** Read palettes, validate roles, generate all supported surfaces consistently, and test opacity boundaries and Night/Storm switching.
4. **Implement installation and restore.** Preview file/package actions, protect existing files, record backups, support host selection, and test interrupted operations and rollback before privileged deployment.
5. **Implement diagnostics and automation.** Inspect packages, GPU/output state, portals, assets, and optional integrations; add CI for Rust and static config validation.

Session-manager integration, real desktop screenshots, and release packaging follow validated behavior. No release date is implied by this list.
