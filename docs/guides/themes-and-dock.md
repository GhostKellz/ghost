# Themes and dock

Night and Storm palettes are in [themes/](../../themes/). The checked-in Lua, GTK CSS, rofi, and lock-screen colors currently use Night. Palette TOML is not read by the desktop at runtime: changing it alone changes no surface. `ghost apply` and automatic opacity rendering remain unimplemented.

## Top bar

Waybar runs one bar per monitor ([config](../../config/waybar/config.jsonc), [style](../../config/waybar/style.css)). It is 36px tall with 15px text, a translucent `alpha(@surface, 0.72)` background blurred by Hyprland, and a hairline bottom border. Modules have no individual boxes; text uses the soft and muted foregrounds, Tokyo Night blue marks the active workspace, and red is reserved for urgent workspaces, muted audio, and a lost network connection.

| Position | Modules | Notes |
|---|---|---|
| Left | Workspaces, focused window | Both scoped to the bar's own monitor; title shows the app icon and truncates at 72 characters |
| Center | Date and time | Hover for a month calendar |
| Right | Media, audio, network, notifications, tray | Media follows Sonora first, then any other MPRIS player (e.g. a browser tab), and is hidden when nothing is playing or paused |

Click actions: scrolling over workspaces switches to the previous/next workspace on that monitor; audio opens `pavucontrol`; network opens `nm-connection-editor`; clicking media toggles play/pause; the bell toggles the swaync panel (right-click toggles do-not-disturb). The bar is deliberately quieter than the dock: no accent border and no hover backgrounds outside the workspace buttons.

## Dock appearance

The dock's [style.css](../../config/nwg-dock-hyprland/style.css) uses `alpha(@surface, 0.55)`, a 16px radius, a faint accent border, and a 150ms blue hover transition. Its base surface color is `#16161e`, defined in the adjacent `colors.css`.

Startup supplies `-d -mb 8 -i 56 -c 'rofi -show drun'`. These request autohide, an eight-pixel bottom margin, 56px icons, and a replacement launcher command. Upstream defaults to bottom placement. `-o` is added only when the host has a `monitor_priority` entry: DP-3 for `arch`, unrestricted output hotspots for the supplied VM/default profiles. See [upstream dock arguments](https://github.com/nwg-piotr/nwg-dock-hyprland/blob/master/README.md).

Tune the deployed files, then restart the dock process or log out and back in. A Hyprland reload does not rerun Ghost's startup hook.

| Desired change | Setting |
|---|---|
| More opaque panel | Increase `0.55` in the dock's `alpha(@surface, ...)` |
| Smaller icons | Lower `-i 56` in `ghost/autostart.lua` |
| Different bottom gap | Change `-mb 8` |
| Different dock monitor | Change the profile's first `monitor_priority` entry |
| Softer or stronger blur | Adjust `decoration.blur` in `ghost/look.lua` |
| Different pinned applications | Edit `nwg-dock-hyprland/pinned` to match installed desktop entries |

Blur is requested for `waybar`, `rofi`, `nwg-dock`, `swaync-control-center`, and `swaync-notification-window`. Confirm the actual namespaces with `hyprctl layers`. The `ignore_alpha = 0.5` threshold means very transparent pixels may not receive blur; lowering panel alpha below that threshold can change the effect. Visual confirmation is still required.

## Wallpapers and manual colors

Deploy one wallpaper set directly to `~/.local/share/ghost/wallpapers/`. `hyprpaper.conf` requests random cycling every half hour. `hyprlock.conf` independently reads `~/.local/share/ghost/lockscreen.png`. See [hyprpaper's documentation](https://wiki.hypr.land/Hypr-Ecosystem/hyprpaper/) for directory cycling support in your installed version.

For a manual Storm conversion, update each application's color file, the Lua palette, and lock-screen colors together using the Storm palette roles. Keep this explicit until the generator exists; comments saying “generated” describe the intended future workflow.

Bundled wallpapers have a [retained MIT license](../../wallpapers/tokyonight/LICENSE.txt). Palette comments attribute colors to `folke/tokyonight.nvim`; provenance and redistribution notices should be checked before a release. Files under `wallpapers/collection/` are personal and gitignored.
