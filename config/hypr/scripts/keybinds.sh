#!/bin/sh
# Lists the running session's keybinds that have a description, in rofi.
# Reads `hyprctl binds -j`, so it always matches what is actually bound.

hyprctl binds -j | jq -r '
    # modmask bits: SHIFT 1, CTRL 4, ALT 8, SUPER 64
    def bit($n): (.modmask / $n | floor) % 2 == 1;
    def mods: [
        (if bit(64) then "SUPER" else empty end),
        (if bit(4) then "CTRL" else empty end),
        (if bit(8) then "ALT" else empty end),
        (if bit(1) then "SHIFT" else empty end)
    ];
    .[]
    | select(.has_description)
    | ((mods + [.key]) | join(" + ")) as $combo
    | $combo + (" " * ([30 - ($combo | length), 2] | max)) + .description
' | rofi -dmenu -i -no-custom -p keybinds >/dev/null
