local M = {}

function M.setup(host)
    hl.env("XCURSOR_SIZE", "24")
    hl.env("HYPRCURSOR_SIZE", "24")
    -- Qt apps use the KDE platform theme (plasma-integration), which applies the
    -- kdeglobals colour scheme below. qt6ct only sets the Qt palette, which KDE
    -- apps such as Dolphin partly ignore (white views).
    hl.env("QT_QPA_PLATFORMTHEME", "kde")
    -- Without this, Dolphin's "Open with" and file associations are empty
    -- outside Plasma (needs archlinux-xdg-menu, then kbuildsycoca6).
    hl.env("XDG_MENU_PREFIX", "arch-")
    -- GTK 3 apps outside GNOME don't reliably pick the theme up from gsettings
    -- or settings.ini on Wayland; name it explicitly. GTK 4/libadwaita apps
    -- ignore themes without a gtk-4.0 directory and use ~/.config/gtk-4.0.
    hl.env("GTK_THEME", "Ghost-TokyoNight")
    hl.env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
    -- KDE apps take colours from kdeglobals, not the Qt palette. Ghost's Tokyo
    -- Night scheme is searched after ~/.config, so a Plasma user's own
    -- kdeglobals still wins and Plasma itself is unaffected.
    local home = os.getenv("HOME")
    if home then
        local kde = home .. "/.config/ghost/kde"
        local dirs = os.getenv("XDG_CONFIG_DIRS") or "/etc/xdg"
        -- The config is re-run on every reload, and hl.env updates this
        -- process's environment, so only prepend once.
        if not (":" .. dirs .. ":"):find(":" .. kde .. ":", 1, true) then
            hl.env("XDG_CONFIG_DIRS", kde .. ":" .. dirs)
        end
    end

    for name, value in pairs(host.env or {}) do
        hl.env(name, value)
    end
end

return M
