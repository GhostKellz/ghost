local M = {}

function M.setup(host)
    -- Dock on the primary monitor only: floating 8px off the edge and hidden
    -- until the pointer reaches the bottom edge (-d), so the OLED panels don't
    -- show a static bar and windows keep the full height. The launcher button
    -- opens rofi (nwg-dock defaults to nwg-drawer, not installed).
    local dock = "nwg-dock-hyprland -d -mb 8 -i 56 -c 'rofi -show drun'"
    local home = os.getenv("HOME")
    if home then
        -- Blue app-grid launcher button instead of the icon theme's default.
        dock = dock .. " -ico " .. home .. "/.config/nwg-dock-hyprland/launcher.svg"
    end
    local primary = host.monitor_priority and host.monitor_priority[1]
    if primary then
        dock = dock .. " -o " .. primary
    end

    hl.on("hyprland.start", function()
        -- Host-specific commands from the profile's optional `startup` list run
        -- first, e.g. creating a headless output the dock is pinned to.
        for _, cmd in ipairs(host.startup or {}) do
            hl.exec_cmd(cmd)
        end
        hl.exec_cmd("waybar")
        hl.exec_cmd("swaync")
        hl.exec_cmd("hypridle")
        hl.exec_cmd("hyprpaper")
        hl.exec_cmd(dock)
        hl.exec_cmd("systemctl --user start hyprpolkitagent")
        hl.exec_cmd("wl-paste --watch cliphist store")
        -- GTK 4/libadwaita apps take dark mode, theme, icons and button layout
        -- from these settings (via the portal), not from settings.ini.
        hl.exec_cmd("gsettings set org.gnome.desktop.interface color-scheme prefer-dark")
        hl.exec_cmd("gsettings set org.gnome.desktop.interface gtk-theme Ghost-TokyoNight")
        hl.exec_cmd("gsettings set org.gnome.desktop.interface icon-theme Tela-blue-dark")
        hl.exec_cmd("gsettings set org.gnome.desktop.wm.preferences button-layout :minimize,maximize,close")
    end)
end

return M
