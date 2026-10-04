# Keybindings

Source: [ghost/binds.lua](../../config/hypr/ghost/binds.lua). `Super` is the Windows/logo key. H/J/K/L correspond to left/down/up/right.

| Binding | Action |
|---|---|
| Super + Return | Ghostty terminal |
| Super + Space | rofi application launcher |
| Super + E | Dolphin |
| Super + W | rofi window switcher |
| Super + V | Clipboard history picker |
| Alt + F4 | Close active window |
| Super + Ctrl + Escape | Force-kill active window |
| Super + F | Toggle floating |
| Super + M | Toggle maximized mode |
| Super + Shift + Return | Swap with master |
| Super + T | Toggle master/dwindle layout |
| Super + H/J/K/L | Focus in that direction |
| Super + Shift + H/J/K/L | Swap windows in that direction |
| Super + Ctrl + H/K | Shrink width/height by 60 |
| Super + Ctrl + L/J | Grow width/height by 60 |
| Super + 1–5 | Select workspace on focused monitor |
| Super + Shift + 1–5 | Move window to workspace without following |
| Super + Ctrl + Left/Right | Previous/next workspace on focused monitor |
| Super + Ctrl + Shift + Left/Right | Carry window to previous/next workspace |
| Super + Shift + Left/Right | Move window to left/right monitor |
| Super + left mouse drag | Move window |
| Super + right mouse drag | Resize window |
| Super + Escape | Request session lock |
| Super + Shift + Escape | Request lock, then blank displays after one second |
| Print | Select region with slurp, annotate with satty |
| Shift + Print | Capture output image with grim, annotate with satty |
| Audio volume keys | Adjust volume in 5% steps, capped at 100% |
| Audio mute / microphone mute | Toggle corresponding default device |
| Media play/pause, previous, next | Control player through playerctl |

The volume and media bindings work while locked. Session locking depends on a running hypridle and functional hyprlock; confirm it manually before relying on it. Workspace semantics depend on whether the optional Lua package is installed; see [configuration](../getting-started/configuration.md).
