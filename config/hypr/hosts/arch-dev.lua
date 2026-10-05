-- Proxmox test VM: virtual bochs display + passed-through RTX 3070 with no
-- monitor attached.
--
-- Left to itself, aquamarine makes the bochs device primary; it has no render
-- node, so every frame is rendered on the 3070 and must cross to bochs, which
-- stalls output entirely. Use only the 3070 (link from the `gpu` component)
-- and render to a headless output, viewed through wayvnc over an SSH tunnel:
--   ssh -N -L 5901:localhost:5900 <user>@<vm>   then VNC to localhost:5901
-- The virtual console keeps showing SDDM and TTYs, not the desktop.
return {
    monitors = {
        { output = "VNC-1", mode = "1920x1080@60", position = "0x0", scale = 1 },
    },
    monitor_priority = { "VNC-1" },
    env = {
        AQ_DRM_DEVICES = "/dev/dri/nvidia-dgpu",
        LIBVA_DRIVER_NAME = "nvidia",
        __GLX_VENDOR_LIBRARY_NAME = "nvidia",
        NVD_BACKEND = "direct",
    },
    -- wayvnc is test tooling on this VM, not a Ghost package. It listens on
    -- localhost only; reach it through the SSH tunnel above.
    startup = {
        "hyprctl output create headless VNC-1 && wayvnc -r -o VNC-1 127.0.0.1",
    },
}
