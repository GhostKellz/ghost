-- Independent workspaces per monitor via the split-monitor-workspaces Lua
-- package (cloned by `ghost install` into plugins/, tag matching Hyprland).
-- If it's missing, fall back to Hyprland's per-monitor selectors so the
-- session still works.

local M = {
    count = 5,
}

local config_dir = (os.getenv("XDG_CONFIG_HOME") or (os.getenv("HOME") .. "/.config")) .. "/hypr"
package.path = package.path .. ";" .. config_dir .. "/plugins/split-monitor-workspaces/lua/?.lua"

local report = require("ghost.report")
local fallback_note = "; using built-in per-monitor workspace switching"

if not package.searchpath("split-monitor-workspaces", package.path) then
    report.warn("split-monitor-workspaces is not installed" .. fallback_note)
else
    local ok, smw = pcall(require, "split-monitor-workspaces")
    if ok and type(smw) == "table" and smw.setup then
        M.smw = smw
    else
        local reason = ok and "unexpected module contents" or tostring(smw)
        report.error("split-monitor-workspaces failed to load (" .. reason .. ")" .. fallback_note)
    end
end

function M.setup(host)
    if not M.smw then
        return
    end
    M.smw.setup({
        workspace_count = M.count,
        monitor_priority = host.monitor_priority,
        keep_focused = true,
        enable_persistent_workspaces = true,
        enable_wrapping = true,
    })
end

-- Nth workspace on the focused monitor.
function M.focus(n)
    if M.smw then
        return M.smw.workspace(tostring(n))
    end
    return hl.dsp.focus({ workspace = "m~" .. n })
end

-- Send the active window to the Nth workspace on the focused monitor.
function M.move_silent(n)
    if M.smw then
        return M.smw.move_to_workspace_silent(tostring(n))
    end
    return hl.dsp.window.move({ workspace = "m~" .. n, follow = false })
end

-- Previous/next workspace on the focused monitor ("prev" | "next").
function M.cycle(direction)
    if M.smw then
        return M.smw.cycle_workspaces(direction)
    end
    return hl.dsp.focus({ workspace = direction == "next" and "m+1" or "m-1" })
end

-- Carry the active window to the previous/next workspace on this monitor.
function M.carry(direction)
    return hl.dsp.window.move({ workspace = direction == "next" and "m+1" or "m-1" })
end

return M
