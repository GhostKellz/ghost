-- Keybinds. Vim-style focus/move/resize and desktop switching keep the
-- KDE + Krohnkite muscle memory; see docs for the full table.

local M = {}

-- Built-in per-monitor selectors, used when the workspaces module failed to
-- load, so workspace keys keep working.
local native_workspaces = {
    count = 5,
    focus = function(n) return hl.dsp.focus({ workspace = "m~" .. n }) end,
    move_silent = function(n) return hl.dsp.window.move({ workspace = "m~" .. n, follow = false }) end,
    cycle = function(d) return hl.dsp.focus({ workspace = d == "next" and "m+1" or "m-1" }) end,
    carry = function(d) return hl.dsp.window.move({ workspace = d == "next" and "m+1" or "m-1" }) end,
}

function M.setup(ws)
    ws = ws or native_workspaces
    local mod = "SUPER"
    local terminal = "ghostty"
    local file_manager = "dolphin"
    local launcher = "rofi -show drun"
    local window_switcher = "rofi -show window"

    local function key(combo)
        return mod .. " + " .. combo
    end

    -- Apps
    hl.bind(key("Return"), hl.dsp.exec_cmd(terminal))
    hl.bind(key("Space"), hl.dsp.exec_cmd(launcher))
    hl.bind(key("E"), hl.dsp.exec_cmd(file_manager))
    -- No overview plugin exists for 0.56 (hyprexpo was removed upstream), so
    -- SUPER+W lists open windows instead.
    hl.bind(key("W"), hl.dsp.exec_cmd(window_switcher))
    hl.bind(key("V"), hl.dsp.exec_cmd("cliphist list | rofi -dmenu -p clipboard | cliphist decode | wl-copy"))

    -- Windows
    hl.bind("ALT + F4", hl.dsp.window.close())
    hl.bind(key("CTRL + Escape"), hl.dsp.window.kill())
    hl.bind(key("F"), hl.dsp.window.float({ action = "toggle" }))
    -- Monocle: maximize within the layout, keeping bar and gaps.
    hl.bind(key("M"), hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }))
    hl.bind(key("SHIFT + Return"), hl.dsp.layout("swapwithmaster master"))

    -- Toggle master <-> dwindle for the whole session.
    hl.bind(key("T"), function()
        local current = hl.get_config("general.layout")
        hl.config({ general = { layout = current == "master" and "dwindle" or "master" } })
    end)

    -- Focus / move / resize (H J K L)
    local directions = { H = "left", J = "down", K = "up", L = "right" }
    for k, dir in pairs(directions) do
        hl.bind(key(k), hl.dsp.focus({ direction = dir }))
        hl.bind(key("SHIFT + " .. k), hl.dsp.window.swap({ direction = dir }))
    end
    -- Same grow/shrink mapping as Krohnkite: L/J grow, H/K shrink.
    local step = 60
    hl.bind(key("CTRL + L"), hl.dsp.window.resize({ x = step, y = 0, relative = true }), { repeating = true })
    hl.bind(key("CTRL + H"), hl.dsp.window.resize({ x = -step, y = 0, relative = true }), { repeating = true })
    hl.bind(key("CTRL + J"), hl.dsp.window.resize({ x = 0, y = step, relative = true }), { repeating = true })
    hl.bind(key("CTRL + K"), hl.dsp.window.resize({ x = 0, y = -step, relative = true }), { repeating = true })

    -- Workspaces (per monitor)
    hl.bind(key("CTRL + left"), ws.cycle("prev"))
    hl.bind(key("CTRL + right"), ws.cycle("next"))
    hl.bind(key("CTRL + SHIFT + left"), ws.carry("prev"))
    hl.bind(key("CTRL + SHIFT + right"), ws.carry("next"))
    for i = 1, ws.count do
        hl.bind(key(tostring(i)), ws.focus(i))
        hl.bind(key("SHIFT + " .. i), ws.move_silent(i))
    end

    -- Monitors
    hl.bind(key("SHIFT + left"), hl.dsp.window.move({ monitor = "l" }))
    hl.bind(key("SHIFT + right"), hl.dsp.window.move({ monitor = "r" }))

    -- Mouse
    hl.bind(key("mouse:272"), hl.dsp.window.drag(), { mouse = true })
    hl.bind(key("mouse:273"), hl.dsp.window.resize(), { mouse = true })

    -- Lock and blank. The delay stops the key release from waking the panels
    -- straight back up (wiki-recommended pattern for dpms binds).
    hl.bind(key("Escape"), hl.dsp.exec_cmd("loginctl lock-session"))
    hl.bind(key("SHIFT + Escape"), function()
        hl.exec_cmd("loginctl lock-session")
        hl.timer(function()
            hl.dispatch(hl.dsp.dpms({ action = "disable" }))
        end, { timeout = 1000, type = "oneshot" })
    end)

    -- Screenshots
    hl.bind("Print", hl.dsp.exec_cmd('grim -g "$(slurp)" - | satty --filename -'))
    hl.bind("SHIFT + Print", hl.dsp.exec_cmd("grim - | satty --filename -"))

    -- Media / volume
    hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
    hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
    hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true })
    hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true })
    hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl --player=sonora,%any play-pause"), { locked = true })
    hl.bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl --player=sonora,%any play-pause"), { locked = true })
    hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl --player=sonora,%any next"), { locked = true })
    hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl --player=sonora,%any previous"), { locked = true })
end

return M
