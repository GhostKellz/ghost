-- On-screen reporting for config problems, so a fallback never looks like a
-- successful load. Uses Hyprland's built-in notifications (not swaync), which
-- work before any session apps have started.

local M = {}

local max_length = 400

local function notify(icon, timeout, message)
    -- Lua load errors span several lines; keep the on-screen text to one.
    local text = ("ghost: " .. tostring(message)):gsub("%s*\n%s*", " | ")
    if #text > max_length then
        text = text:sub(1, max_length) .. "…"
    end
    if hl.notification and hl.notification.create then
        hl.notification.create({ text = text, timeout = timeout, icon = icon })
    end
    print(text)
end

function M.error(message)
    notify("error", 20000, message)
end

function M.warn(message)
    notify("warning", 10000, message)
end

return M
