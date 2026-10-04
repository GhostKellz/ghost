-- Workstation: RTX 5090 + Ryzen iGPU, two ASUS 4K 240 Hz QD-OLEDs.
--
-- Outputs are matched by connector for now; switch to `desc:` strings from
-- `hyprctl monitors` once confirmed on this machine (connectors moved when the
-- second panel was swapped in).
local hdr = {
    bitdepth = 10,
    -- "hdr" is marked experimental upstream; fall back to cm = "auto" here if
    -- the desktop misbehaves. Fullscreen HDR also works via render.cm_auto_hdr.
    cm = "hdr",
    sdrbrightness = 1.0,
    sdrsaturation = 1.0,
}

local function monitor(output, position)
    local rule = { output = output, mode = "3840x2160@240", position = position, scale = 1 }
    for k, v in pairs(hdr) do
        rule[k] = v
    end
    return rule
end

return {
    monitors = {
        monitor("DP-3", "0x0"), -- PG32UCDM3, primary, left
        monitor("DP-2", "3840x0"), -- PG32UCDM, right
    },
    monitor_priority = { "DP-3", "DP-2" },
    secondary_monitor = "DP-2",
    env = {
        -- The iGPU is card0; render and scan out on the 5090. The symlink comes
        -- from system/udev/61-ghost-nvidia-dgpu.rules (card numbers can swap).
        AQ_DRM_DEVICES = "/dev/dri/nvidia-dgpu",
        LIBVA_DRIVER_NAME = "nvidia",
        __GLX_VENDOR_LIBRARY_NAME = "nvidia",
        NVD_BACKEND = "direct",
    },
}
