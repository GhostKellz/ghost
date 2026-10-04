-- Active palette. Mirrors themes/tokyonight-night/palette.toml; `ghost apply
-- --theme <name>` will regenerate this file from the selected palette.toml.

local M = {
    name = "tokyonight-night",
    colors = {
        bg = "1a1b26",
        bg_dark = "16161e",
        bg_highlight = "292e42",
        fg = "c0caf5",
        comment = "565f89",
        terminal_black = "414868",
        blue = "7aa2f7",
        cyan = "7dcfff",
        green = "9ece6a",
        magenta = "bb9af7",
        orange = "ff9e64",
        red = "f7768e",
        yellow = "e0af68",
    },
}

-- Hyprland color string from a palette key and alpha (0-255).
function M.rgba(key, alpha)
    return string.format("rgba(%s%02x)", M.colors[key], alpha or 255)
end

return M
