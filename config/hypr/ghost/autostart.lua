local M = {}

function M.setup(host)
    -- Dock on the primary monitor only: floating 8px off the edge, auto-hide,
    -- launcher button opens rofi (nwg-dock defaults to nwg-drawer, not installed).
    local dock = "nwg-dock-hyprland -d -mb 8 -i 56 -c 'rofi -show drun'"
    local primary = host.monitor_priority and host.monitor_priority[1]
    if primary then
        dock = dock .. " -o " .. primary
    end

    hl.on("hyprland.start", function()
        hl.exec_cmd("waybar")
        hl.exec_cmd("swaync")
        hl.exec_cmd("hypridle")
        hl.exec_cmd("hyprpaper")
        hl.exec_cmd(dock)
        hl.exec_cmd("systemctl --user start hyprpolkitagent")
        hl.exec_cmd("wl-paste --watch cliphist store")
    end)
end

return M
