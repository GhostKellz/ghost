local M = {}

function M.setup()
    local palette = require("ghost.palette")

    hl.config({
        general = {
            -- Small gaps, matching the KDE/Krohnkite padding of 4.
            gaps_in = 2,
            gaps_out = 4,
            border_size = 2,
            col = {
                active_border = {
                    colors = { palette.rgba("blue"), palette.rgba("magenta") },
                    angle = 45,
                },
                inactive_border = palette.rgba("terminal_black", 0xaa),
            },
            -- Master+stack mirrors Krohnkite's Tile layout; SUPER+T toggles dwindle.
            layout = "master",
            resize_on_border = true,
        },
        decoration = {
            rounding = 8,
            active_opacity = 1.0,
            inactive_opacity = 1.0,
            shadow = {
                enabled = true,
                range = 8,
                render_power = 3,
                color = palette.rgba("bg_dark", 0xee),
            },
            blur = {
                enabled = true,
                size = 6,
                passes = 2,
                vibrancy = 0.17,
            },
        },
        master = {
            new_status = "master",
            mfact = 0.55,
        },
        dwindle = {
            preserve_split = true,
        },
        misc = {
            -- VRR only for fullscreen game/video content: desktop VRR flickers on OLED.
            vrr = 3,
            force_default_wallpaper = 0,
            disable_hyprland_logo = true,
            disable_splash_rendering = true,
        },
        xwayland = {
            force_zero_scaling = true,
        },
    })
end

return M
