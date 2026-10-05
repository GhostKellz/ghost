-- Keybinds. Vim-style focus/move/resize and desktop switching keep the
-- KDE + Krohnkite muscle memory; see docs for the full table.
--
-- Every bind carries a description: SUPER+/ lists them from `hyprctl binds`,
-- where Lua binds otherwise show only as "__lua", so the help can't drift
-- from the real bindings.

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

local function bind(combo, description, action, opts)
    opts = opts or {}
    opts.description = description
    hl.bind(combo, action, opts)
end

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
    bind(key("Return"), "Terminal", hl.dsp.exec_cmd(terminal))
    bind(key("Space"), "Application launcher", hl.dsp.exec_cmd(launcher))
    bind(key("E"), "File manager", hl.dsp.exec_cmd(file_manager))
    -- No overview plugin exists for 0.56 (hyprexpo was removed upstream), so
    -- SUPER+W lists open windows instead.
    bind(key("W"), "Window switcher", hl.dsp.exec_cmd(window_switcher))
    bind(key("V"), "Clipboard history", hl.dsp.exec_cmd("cliphist list | rofi -dmenu -p clipboard | cliphist decode | wl-copy"))
    bind(key("slash"), "Keybind help", hl.dsp.exec_cmd("~/.config/hypr/scripts/keybinds.sh"))

    -- Windows
    bind(key("Q"), "Close window", hl.dsp.window.close())
    bind("ALT + F4", "Close window", hl.dsp.window.close())
    bind(key("CTRL + Escape"), "Force-kill window", hl.dsp.window.kill())
    bind(key("F"), "Toggle floating", hl.dsp.window.float({ action = "toggle" }))
    -- Monocle: maximize within the layout, keeping bar and gaps.
    bind(key("M"), "Toggle maximized", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }))
    bind(key("SHIFT + Return"), "Swap with master", hl.dsp.layout("swapwithmaster master"))

    -- Toggle master <-> dwindle for the whole session.
    bind(key("T"), "Toggle master/dwindle layout", function()
        local current = hl.get_config("general.layout")
        hl.config({ general = { layout = current == "master" and "dwindle" or "master" } })
    end)

    -- Focus / move / resize (H J K L)
    local directions = { H = "left", J = "down", K = "up", L = "right" }
    for k, dir in pairs(directions) do
        bind(key(k), "Focus " .. dir, hl.dsp.focus({ direction = dir }))
        bind(key("SHIFT + " .. k), "Swap window " .. dir, hl.dsp.window.swap({ direction = dir }))
    end
    -- Same grow/shrink mapping as Krohnkite: L/J grow, H/K shrink.
    local step = 60
    bind(key("CTRL + L"), "Grow width", hl.dsp.window.resize({ x = step, y = 0, relative = true }), { repeating = true })
    bind(key("CTRL + H"), "Shrink width", hl.dsp.window.resize({ x = -step, y = 0, relative = true }), { repeating = true })
    bind(key("CTRL + J"), "Grow height", hl.dsp.window.resize({ x = 0, y = step, relative = true }), { repeating = true })
    bind(key("CTRL + K"), "Shrink height", hl.dsp.window.resize({ x = 0, y = -step, relative = true }), { repeating = true })

    -- Workspaces (per monitor)
    bind(key("CTRL + left"), "Previous workspace", ws.cycle("prev"))
    bind(key("CTRL + right"), "Next workspace", ws.cycle("next"))
    bind(key("CTRL + SHIFT + left"), "Carry window to previous workspace", ws.carry("prev"))
    bind(key("CTRL + SHIFT + right"), "Carry window to next workspace", ws.carry("next"))
    for i = 1, ws.count do
        bind(key(tostring(i)), "Workspace " .. i, ws.focus(i))
        bind(key("SHIFT + " .. i), "Move window to workspace " .. i, ws.move_silent(i))
    end

    -- Monitors
    bind(key("SHIFT + left"), "Move window to left monitor", hl.dsp.window.move({ monitor = "l" }))
    bind(key("SHIFT + right"), "Move window to right monitor", hl.dsp.window.move({ monitor = "r" }))

    -- Mouse
    bind(key("mouse:272"), "Move window (drag)", hl.dsp.window.drag(), { mouse = true })
    bind(key("mouse:273"), "Resize window (drag)", hl.dsp.window.resize(), { mouse = true })

    -- Lock and blank. The delay stops the key release from waking the panels
    -- straight back up (wiki-recommended pattern for dpms binds).
    bind(key("Escape"), "Lock session", hl.dsp.exec_cmd("loginctl lock-session"))
    bind(key("SHIFT + Escape"), "Lock and blank displays", function()
        hl.exec_cmd("loginctl lock-session")
        hl.timer(function()
            hl.dispatch(hl.dsp.dpms({ action = "disable" }))
        end, { timeout = 1000, type = "oneshot" })
    end)

    -- Screenshots
    bind("Print", "Screenshot region", hl.dsp.exec_cmd('grim -g "$(slurp)" - | satty --filename -'))
    bind("SHIFT + Print", "Screenshot all outputs", hl.dsp.exec_cmd("grim - | satty --filename -"))

    -- Media / volume
    bind("XF86AudioRaiseVolume", "Volume up", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
    bind("XF86AudioLowerVolume", "Volume down", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
    bind("XF86AudioMute", "Mute output", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true })
    bind("XF86AudioMicMute", "Mute microphone", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true })
    bind("XF86AudioPlay", "Play/pause", hl.dsp.exec_cmd("playerctl --player=sonora,%any play-pause"), { locked = true })
    bind("XF86AudioPause", "Play/pause", hl.dsp.exec_cmd("playerctl --player=sonora,%any play-pause"), { locked = true })
    bind("XF86AudioNext", "Next track", hl.dsp.exec_cmd("playerctl --player=sonora,%any next"), { locked = true })
    bind("XF86AudioPrev", "Previous track", hl.dsp.exec_cmd("playerctl --player=sonora,%any previous"), { locked = true })
end

return M
