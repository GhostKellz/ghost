#!/bin/sh
# Waybar media module. Follows the preferred MPRIS player (Sonora first, then
# any) and prints one JSON line per change. Empty text when nothing is playing
# or paused, which makes Waybar hide the module regardless of Waybar version.

players="sonora,%any"
play_icon=$(printf '\357\201\213')  # U+F04B
pause_icon=$(printf '\357\201\214') # U+F04C
# Unit separator: unlike tab it isn't IFS whitespace, so empty fields survive.
sep=$(printf '\037')

# JSON string escaping for the fields we emit.
json() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

playerctl --player="$players" --follow metadata \
    --format "{{status}}${sep}{{playerName}}${sep}{{artist}}${sep}{{title}}" 2>/dev/null |
while IFS="$sep" read -r status player artist title; do
    case "$status" in
        Playing) icon=$play_icon class=playing ;;
        Paused) icon=$pause_icon class=paused ;;
        *)
            printf '{"text":""}\n'
            continue
            ;;
    esac

    if [ -n "$artist" ]; then
        track="$artist – $title"
    else
        track="$title"
    fi

    printf '{"text":"%s  %s","tooltip":"%s: %s","class":"%s"}\n' \
        "$icon" "$(json "$track")" "$(json "$player")" "$(json "$track")" "$class"
done
