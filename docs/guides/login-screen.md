# Login screen

Ghost includes an SDDM theme in [system/sddm/ghost/](../../system/sddm/ghost/): a translucent Tokyo Night panel with power controls, the Ghost logo, password entry, and a session selector. SDDM performs authentication and starts the session; the theme only calls SDDM's `login`, `powerOff`, `reboot`, and `suspend`.

Requirements: SDDM 0.21 with its Qt 6 greeter (`sddm-greeter-qt6`) and `qt6-declarative`, which provides QtQuick Controls, Effects, and Shapes. Nothing in this guide changes the active theme until the activation step.

## Behavior

| Area | Behavior |
|---|---|
| Screens | The panel appears on the primary screen; other screens show the background only |
| User | The last-used user is preselected; arrows appear when more than one user is listed; a username field appears when SDDM lists no users |
| Keyboard | Tab order: password, Sign in, Session, Suspend, Restart, Shut down. Enter signs in |
| Errors | A failed login clears the password, shows a red message, and keeps focus in the field. PAM information messages and Caps Lock state use the same line |
| Power | Suspend is immediate. Restart and Shut down ask for confirmation; Cancel is focused and Esc dismisses. Buttons SDDM reports as unavailable are hidden |
| Glass | The panel blurs the wallpaper (or the default glow) behind it, with a lit top edge and a soft shadow. Blur needs a GPU-backed Qt scene graph; under software rendering the panel is translucent without blur |

## Configure

Defaults are in [theme.conf](../../system/sddm/ghost/theme.conf). Override them in `theme.conf.user` next to it in the installed theme; SDDM reads that file over `theme.conf`.

| Key | Default | Meaning |
|---|---|---|
| `background` | empty | Wallpaper, absolute or relative to the theme directory |
| `backgroundColor` | `#1a1b26` | Base colour |
| `glow` | `true` | Without a wallpaper, draw soft blue and cyan light over the base colour so the glass panel has something to frost; `false` gives a flat colour |
| `avatar` | `logo` | `user` shows the account picture, falling back to the logo when none is set |
| `logo` | `assets/logo.png` | Image in the avatar circle |
| `accent`, `accentAlt` | `#7aa2f7`, `#7dcfff` | Button, ring, and focus colors |
| `panelOpacity` | `0.55` | Panel fill opacity; lower is more glass-like |
| `blur` | `64` | Maximum blur radius in pixels; `0` disables it |
| `font` | `Noto Sans` | Font family |

Store your wallpaper inside the installed theme, for example `backgrounds/wallpaper.jpg`. The `sddm` user usually cannot read home directories. Ghost ships no wallpaper; `system/sddm/ghost/backgrounds/` is gitignored for local copies.

## Install

`ghost install --with sddm` installs the `sddm` package and the theme, and selects the theme with the drop-in described under [Activate](#activate). It backs up a drop-in that already exists, and `ghost restore` removes the theme again. It doesn't enable `sddm.service`, so another display manager stays in charge until you switch it.

To install by hand instead, run this from the repository root. It copies the theme and logo, so the installed theme doesn't depend on the checkout:

```bash
sudo install -d /usr/share/sddm/themes/ghost/assets
sudo install -m 644 system/sddm/ghost/*.qml system/sddm/ghost/metadata.desktop \
    system/sddm/ghost/theme.conf /usr/share/sddm/themes/ghost/
sudo install -m 644 assets/logo.png /usr/share/sddm/themes/ghost/assets/logo.png
```

## Preview

Test mode opens the theme in a window with mock users and sessions. It cannot authenticate, and its power buttons do nothing:

```bash
sddm-greeter-qt6 --test-mode --theme /usr/share/sddm/themes/ghost
```

## Activate

Check which file currently sets the theme:

```bash
grep -rs '^Current=' /etc/sddm.conf /etc/sddm.conf.d/
```

SDDM reads `/etc/sddm.conf.d/` in alphabetical order, so later files override earlier ones. Plasma writes `kde_settings.conf`, so the Ghost drop-in needs a name that sorts after it. If `/etc/sddm.conf` sets `Current=`, change it there instead:

```ini
# /etc/sddm.conf.d/zz-ghost.conf
[Theme]
Current=ghost
```

The change applies the next time SDDM starts. Restarting SDDM ends every graphical session on the machine.

## Roll back

Remove `/etc/sddm.conf.d/zz-ghost.conf`, or restore the previous `Current=` value. The previous theme returns the next time SDDM starts. To uninstall, also remove `/usr/share/sddm/themes/ghost`.
