-- Proxmox test VM: virtual bochs display + passed-through RTX 3070.
--
-- Monitors use the fallback rule until it's known which outputs the VM
-- actually exposes (bochs console only, or a display/dummy plug on the 3070).
return {
    monitors = {},
    env = {
        LIBVA_DRIVER_NAME = "nvidia",
        __GLX_VENDOR_LIBRARY_NAME = "nvidia",
        NVD_BACKEND = "direct",
    },
}
