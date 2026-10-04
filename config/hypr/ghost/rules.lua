-- Window and layer rules. Class names must match `hyprctl clients` output;
-- verify them on first run.

local M = {}

function M.setup(host)
    -- Password manager and hardware wallet: small, centered, never tiled.
    for _, class in ipairs({ "^(Bitwarden)$", "^(Ledger Live)$" }) do
        hl.window_rule({
            match = { class = class },
            float = true,
            center = true,
            size = { "monitor_w*0.4", "monitor_h*0.6" },
        })
    end

    hl.window_rule({ match = { class = "^(steam)$", title = "negative:^(Steam)$" }, float = true })
    hl.window_rule({ match = { title = "^(Picture-in-Picture)$" }, float = true, pin = true })

    -- Chat, music, and Steam live on the secondary monitor so the primary
    -- stays for work.
    if host.secondary_monitor then
        for _, class in ipairs({ "^(discord)$", "^(sonora)$", "^(steam)$" }) do
            hl.window_rule({ match = { class = class }, monitor = host.secondary_monitor })
        end
    end

    -- Blur behind translucent shell surfaces.
    for _, namespace in ipairs({ "waybar", "rofi", "nwg-dock", "swaync-control-center", "swaync-notification-window" }) do
        hl.layer_rule({ match = { namespace = namespace }, blur = true, ignore_alpha = 0.5 })
    end
end

return M
