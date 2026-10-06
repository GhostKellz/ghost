# Themes and dock

Night and Storm palettes are in [themes/](../../themes/). The checked-in Lua, GTK CSS, KDE colour scheme, rofi, and lock-screen colors currently use Night. Palette TOML is not read by the desktop at runtime: changing it alone changes no surface. `ghost apply` and automatic opacity rendering remain unimplemented.

## Top bar

Waybar runs one bar per monitor ([config](../../config/waybar/config.jsonc), [style](../../config/waybar/style.css)). It is 36px tall with 15px text, a translucent `alpha(@surface, 0.40)` background blurred by Hyprland, and a hairline bottom border. Text uses the soft and muted foregrounds, and red is reserved for urgent workspaces, muted audio, and a lost network connection.

| Position | Modules | Notes |
|---|---|---|
| Left | Workspace pager, focused window | One box per workspace on the bar's own monitor: active is filled blue, occupied is brighter, empty is a faint outline. The title shows the app icon and truncates at 72 characters |
| Center | Date and time | Hover for a month calendar |
| Right | Media, audio, network, notifications, tray | Media follows Sonora first, then any other MPRIS player (e.g. a browser tab), and is hidden when nothing is playing or paused |

Click actions: scrolling over the pager switches to the previous/next workspace on that monitor; audio opens `pavucontrol`; network opens `nm-connection-editor`; clicking media toggles play/pause; the bell toggles the swaync panel (right-click toggles do-not-disturb).

## Dock

The dock's [style.css](../../config/nwg-dock-hyprland/style.css) uses `alpha(@surface, 0.40)`, a 16px radius, a faint accent border, no background behind individual icons, and a soft blue glow on hover. Its base surface color is `#16161e`, defined in the adjacent `colors.css`.

Startup supplies `-d -mb 8 -i 56 -c 'rofi -show drun' -ico ~/.config/nwg-dock-hyprland/launcher.svg`: hidden until the pointer reaches the bottom edge, an eight-pixel bottom margin, 56px icons, rofi as the launcher, and a blue app-grid launcher button. Auto-hide keeps a static element off OLED panels and gives windows the full height. `-o` is added only when the host has a `monitor_priority` entry. See [upstream dock arguments](https://github.com/nwg-piotr/nwg-dock-hyprland/blob/master/README.md).

Tune the deployed files, then restart the dock process or log out and back in. A Hyprland reload does not rerun Ghost's startup hook.

| Desired change | Setting |
|---|---|
| Always visible, windows tiled above it | Replace `-d` with `-r -x` in `ghost/autostart.lua` |
| More opaque panel | Increase `0.40` in the dock's `alpha(@surface, ...)` |
| Smaller icons | Lower `-i 56` in `ghost/autostart.lua` |
| Different bottom gap | Change `-mb 8` |
| Different dock monitor | Change the profile's first `monitor_priority` entry |
| Softer or stronger blur | Adjust `decoration.blur` in `ghost/look.lua` |
| Different pinned applications | Edit `config/nwg-dock-hyprland/pinned` (desktop entry IDs); Ghost installs it to `~/.cache/nwg-dock-pinned`, the only path nwg-dock reads |

Blur is requested for `waybar`, `rofi`, `nwg-dock`, `swaync-control-center`, and `swaync-notification-window`, with `ignore_alpha = 0.1` so the translucent bar and dock still qualify. Confirm the namespaces with `hyprctl layers`.

## Applications

The `apps` component themes regular applications in Tokyo Night:

| Toolkit | How | Files |
|---|---|---|
| GTK 3 | `Ghost-TokyoNight` theme: Adwaita-dark plus Tokyo Night colours, including unfocused windows. Shipped as a theme rather than `~/.config/gtk-3.0/gtk.css` so the dock's and Waybar's own CSS still take precedence. Selected by `GTK_THEME` in the session environment | [themes/Ghost-TokyoNight](../../config/themes/Ghost-TokyoNight/), [gtk-3.0/settings.ini](../../config/gtk-3.0/settings.ini) |
| GTK 4 / libadwaita | Dark colour scheme (gsettings at startup) plus Tokyo Night colour variables | [gtk-4.0/gtk.css](../../config/gtk-4.0/gtk.css) |
| Qt / KDE (Dolphin, Ark) | `QT_QPA_PLATFORMTHEME=kde` (plasma-integration) and a Tokyo Night `kdeglobals` found through `XDG_CONFIG_DIRS`. A Plasma user's own `~/.config/kdeglobals` still wins, so Plasma sessions are unaffected | [kde/kdeglobals](../../config/kde/kdeglobals) |
| Ghostty | Hacker-blue text (`#57c7ff`) on `#0d1117` at 82% opacity, blurred by Hyprland; CaskaydiaCove Nerd Font | [ghostty/config](../../config/ghostty/config) |
| Yazi | Same hacker-blue palette; image, video, PDF, and archive previews inside Ghostty | [yazi/theme.toml](../../config/yazi/theme.toml) |

GTK apps that draw their own title bar show macOS-style buttons on the right: red close, yellow minimise, green maximise. Most windows have no title bar under Hyprland; use the [keybindings](../reference/keybindings.md) instead.

On a machine that also runs Plasma, note that `~/.config/gtk-3.0/settings.ini` is shared with Plasma's own GTK settings.

## Icons

GTK and KDE settings name the `Tela-blue-dark` icon theme. The `apps` component installs it per user into `~/.local/share/icons` from [vinceliuice/Tela-icon-theme](https://github.com/vinceliuice/Tela-icon-theme) at a pinned commit (see [Sources](../reference/cli.md#sources)). Without it, apps fall back to the default icons. Ghostty's launcher entry is overridden to keep Ghostty's own icon.

## Launcher entries

[config/applications/](../../config/applications/) hides developer and system tools pulled in as dependencies (Qt tools, Avahi browsers, `lstopo`, rofi's own entries, V4L2 test tools) from rofi. Each file is a per-user `NoDisplay=true` override in `~/.local/share/applications`; nothing is uninstalled. Delete a file there to show that entry again.

## Wallpapers

Ghost ships no wallpapers; you provide your own.

| Path | Used by | Notes |
|---|---|---|
| `~/.local/share/ghost/wallpapers/` | hyprpaper | Images in this folder are shown in random order, changing every 30 minutes |
| `~/.local/share/ghost/lockscreen.png` | hyprlock | Lock-screen background |

Add or remove images in the folder at any time; restart hyprpaper (or log out and back in) to pick up changes. Change the interval with `timeout` in `hypr/hyprpaper.conf`. The login screen has its own wallpaper setting; see [login screen](login-screen.md).

## Manual colors

For a manual Storm conversion, update each application's color file, the Lua palette, the GTK theme, `kdeglobals`, and lock-screen colors together using the Storm palette roles. Keep this explicit until the generator exists.

Palette comments attribute colors to `folke/tokyonight.nvim`; provenance and redistribution notices should be checked before a release.
