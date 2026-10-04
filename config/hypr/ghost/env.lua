local M = {}

function M.setup(host)
    hl.env("XCURSOR_SIZE", "24")
    hl.env("HYPRCURSOR_SIZE", "24")
    -- Qt apps (Dolphin) take their theme from qt6ct + Kvantum outside Plasma.
    hl.env("QT_QPA_PLATFORMTHEME", "qt6ct")
    -- Without this, Dolphin's "Open with" and file associations are empty
    -- outside Plasma (needs archlinux-xdg-menu, then kbuildsycoca6).
    hl.env("XDG_MENU_PREFIX", "arch-")
    hl.env("ELECTRON_OZONE_PLATFORM_HINT", "auto")

    for name, value in pairs(host.env or {}) do
        hl.env(name, value)
    end
end

return M
