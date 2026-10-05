use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;

use crate::manifest::{self, Manifest};
use crate::state::State;
use crate::system::{Gpu, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Info,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub area: &'static str,
    pub status: Status,
    pub summary: String,
    pub fix: Option<String>,
}

impl Check {
    fn new(area: &'static str, status: Status, summary: impl Into<String>) -> Self {
        Self {
            area,
            status,
            summary: summary.into(),
            fix: None,
        }
    }

    fn fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}

/// Runs every check. Nothing here changes the system.
pub fn run(sys: &System) -> Vec<Check> {
    let mut checks = Vec::new();
    checks.extend(system(sys));
    checks.extend(packages(sys));
    checks.extend(gpu(sys));
    checks.extend(config(sys));
    checks.extend(session(sys));
    checks
}

fn system(sys: &System) -> Vec<Check> {
    const AREA: &str = "System";
    let mut checks = Vec::new();

    if sys.exists("/etc/arch-release") {
        checks.push(Check::new(AREA, Status::Ok, "Arch Linux"));
    } else {
        checks.push(Check::new(
            AREA,
            Status::Fail,
            "Not Arch Linux (/etc/arch-release missing)",
        ));
    }

    // After a kernel upgrade without a reboot, the running kernel's module
    // directory is gone: anything not already loaded (USB, filesystems, DKMS
    // modules) fails to load until the next boot.
    if let Some(release) = sys.read("/proc/sys/kernel/osrelease") {
        if sys.exists(&format!("/usr/lib/modules/{release}")) {
            checks.push(Check::new(AREA, Status::Ok, format!("Kernel {release}")));
        } else {
            checks.push(
                Check::new(
                    AREA,
                    Status::Warn,
                    format!(
                        "Reboot pending: modules for running kernel {release} are not installed"
                    ),
                )
                .fix("Reboot into the installed kernel"),
            );
        }
    }
    checks
}

fn packages(sys: &System) -> Vec<Check> {
    const AREA: &str = "Packages";
    let Some(list) = sys.run("pacman", &["-Qq"]) else {
        return vec![Check::new(
            AREA,
            Status::Warn,
            "Could not run pacman; package checks skipped",
        )];
    };
    let installed: HashSet<&str> = list.lines().map(str::trim).collect();
    let Ok(manifest) = Manifest::parse(manifest::EMBEDDED) else {
        return vec![Check::new(
            AREA,
            Status::Fail,
            "Embedded ghost.toml is invalid",
        )];
    };
    // Packages for the components Ghost deployed, or the defaults before the first install.
    let deployed = State::load(&sys.state_dir.join("state.json"))
        .map(|state| state.components)
        .unwrap_or_default();
    let mut components: Vec<String> = deployed
        .into_iter()
        .filter(|name| manifest.components.contains_key(name))
        .collect();
    if components.is_empty() {
        components = manifest.select(&[], &[]).unwrap_or_default();
    }
    let wanted = manifest.packages(&components);
    let missing: Vec<&str> = wanted
        .iter()
        .copied()
        .filter(|name| !installed.contains(name))
        .collect();

    let mut checks = Vec::new();
    let scope = format!("for {}", components.join(", "));
    if missing.is_empty() {
        checks.push(Check::new(
            AREA,
            Status::Ok,
            format!("All {} packages {scope} installed", wanted.len()),
        ));
    } else {
        checks.push(
            Check::new(
                AREA,
                Status::Fail,
                format!(
                    "{} of {} packages {scope} missing: {}",
                    missing.len(),
                    wanted.len(),
                    missing.join(" ")
                ),
            )
            .fix(format!("sudo pacman -S --needed {}", missing.join(" "))),
        );
    }

    let sessions = ["hyprland.desktop", "hyprland-uwsm.desktop"];
    if sessions
        .iter()
        .any(|file| sys.exists(&format!("/usr/share/wayland-sessions/{file}")))
    {
        checks.push(Check::new(
            AREA,
            Status::Ok,
            "Hyprland login session installed",
        ));
    } else {
        checks.push(Check::new(
            AREA,
            Status::Warn,
            "No Hyprland entry in /usr/share/wayland-sessions",
        ));
    }
    checks
}

fn gpu(sys: &System) -> Vec<Check> {
    const AREA: &str = "GPU";
    let gpus = sys.gpus();
    if gpus.is_empty() {
        return vec![Check::new(
            AREA,
            Status::Warn,
            "No DRM devices found in /sys/class/drm",
        )];
    }

    let mut checks: Vec<Check> = gpus
        .iter()
        .map(|g| {
            Check::new(
                AREA,
                Status::Info,
                format!(
                    "{}: vendor {} driver {}",
                    g.card,
                    g.vendor,
                    g.driver.as_deref().unwrap_or("none")
                ),
            )
        })
        .collect();

    let nvidia: Vec<&Gpu> = gpus.iter().filter(|g| g.is_nvidia()).collect();
    if nvidia.is_empty() {
        return checks;
    }

    // With more than one GPU, Hyprland renders on whichever card aquamarine
    // picks first unless AQ_DRM_DEVICES names one. Card numbers can swap
    // between boots, so Ghost's profiles use a udev-provided stable path.
    if gpus.len() > 1 {
        if sys.exists("/dev/dri/nvidia-dgpu") {
            checks.push(Check::new(
                AREA,
                Status::Ok,
                "Stable /dev/dri/nvidia-dgpu link present for AQ_DRM_DEVICES",
            ));
        } else {
            checks.push(
                Check::new(AREA, Status::Warn, format!("{} GPUs and no /dev/dri/nvidia-dgpu link; a profile setting AQ_DRM_DEVICES to it will not start", gpus.len()))
                    .fix("sudo install -m 644 system/udev/61-ghost-nvidia-dgpu.rules /etc/udev/rules.d/ && sudo udevadm control --reload && sudo udevadm trigger --subsystem-match=drm"),
            );
        }
    }
    checks.extend(nvidia_driver(sys, &nvidia));
    checks
}

fn nvidia_driver(sys: &System, cards: &[&Gpu]) -> Vec<Check> {
    const AREA: &str = "NVIDIA";
    let mut checks = Vec::new();

    for card in cards {
        if card.driver.as_deref() != Some("nvidia") {
            checks.push(Check::new(
                AREA,
                Status::Fail,
                format!(
                    "{} is bound to {}, not the nvidia driver",
                    card.card,
                    card.driver.as_deref().unwrap_or("no driver")
                ),
            ));
        }
    }

    let open = match sys.read("/proc/driver/nvidia/version") {
        Some(version) => {
            let first = version.lines().next().unwrap_or_default();
            let open = first.contains("Open Kernel Module");
            let kind = if open {
                "open kernel modules"
            } else {
                "proprietary kernel module"
            };
            let number = first.split_whitespace().find(|word| {
                word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.')
            });
            checks.push(Check::new(
                AREA,
                Status::Ok,
                format!("Driver {} ({kind})", number.unwrap_or("unknown version")),
            ));
            open
        }
        None => {
            checks.push(
                Check::new(AREA, Status::Fail, "NVIDIA kernel module not loaded").fix(
                    "Install nvidia-open-dkms (or your kernel's nvidia-open package) and reboot",
                ),
            );
            return checks;
        }
    };

    match sys
        .read("/sys/module/nvidia_drm/parameters/modeset")
        .as_deref()
    {
        Some("Y") => checks.push(Check::new(AREA, Status::Ok, "nvidia_drm modeset enabled")),
        _ => checks.push(
            Check::new(
                AREA,
                Status::Fail,
                "nvidia_drm modeset is off; Wayland compositors need it",
            )
            // nvidia-utils enables modeset and fbdev by default, so "off"
            // means something explicitly disabled them.
            .fix("Remove nvidia_drm modeset=0 from /etc/modprobe.d or the kernel command line, then reboot"),
        ),
    }
    match sys
        .read("/sys/module/nvidia_drm/parameters/fbdev")
        .as_deref()
    {
        Some("Y") => checks.push(Check::new(AREA, Status::Ok, "nvidia_drm fbdev enabled")),
        _ => checks.push(
            Check::new(AREA, Status::Warn, "nvidia_drm fbdev is off")
                .fix("Remove nvidia_drm fbdev=0 from /etc/modprobe.d or the kernel command line, then reboot"),
        ),
    }

    checks.extend(suspend(sys, open));
    checks
}

const SUSPEND_SERVICES: [&str; 3] = [
    "nvidia-suspend.service",
    "nvidia-resume.service",
    "nvidia-hibernate.service",
];

/// Video memory across suspend. With the open modules, kernel suspend
/// notifiers (set by nvidia-utils since 595) handle it; NVIDIA documents the
/// systemd services and PreserveVideoMemoryAllocations for the proprietary
/// module and older drivers.
fn suspend(sys: &System, open: bool) -> Vec<Check> {
    const AREA: &str = "NVIDIA";
    let params = sys.read("/proc/driver/nvidia/params").unwrap_or_default();
    let enabled: Vec<&str> = SUSPEND_SERVICES
        .iter()
        .copied()
        .filter(|unit| {
            sys.run("systemctl", &["is-enabled", unit])
                .is_some_and(|state| state.trim() == "enabled")
        })
        .collect();

    if open && param(&params, "UseKernelSuspendNotifiers") == Some("1") {
        let mut checks = vec![Check::new(
            AREA,
            Status::Ok,
            "Kernel suspend notifiers preserve video memory",
        )];
        if !enabled.is_empty() {
            checks.push(Check::new(
                AREA,
                Status::Info,
                format!(
                    "Also enabled: {}; NVIDIA documents these for the proprietary module, not needed here",
                    enabled.join(" ")
                ),
            ));
        }
        return checks;
    }

    let mut checks = Vec::new();
    match param(&params, "PreserveVideoMemoryAllocations") {
        Some("1") => checks.push(Check::new(
            AREA,
            Status::Ok,
            "PreserveVideoMemoryAllocations = 1",
        )),
        value => checks.push(
            Check::new(
                AREA,
                Status::Warn,
                format!(
                    "PreserveVideoMemoryAllocations = {}; suspend may lose video memory",
                    value.unwrap_or("unset")
                ),
            )
            .fix("Set options nvidia NVreg_PreserveVideoMemoryAllocations=1 in /etc/modprobe.d, then reboot"),
        ),
    }
    let disabled: Vec<&str> = SUSPEND_SERVICES
        .iter()
        .copied()
        .filter(|unit| !enabled.contains(unit))
        .collect();
    if disabled.is_empty() {
        checks.push(Check::new(
            AREA,
            Status::Ok,
            "Suspend/resume/hibernate services enabled",
        ));
    } else {
        checks.push(
            Check::new(
                AREA,
                Status::Warn,
                format!(
                    "Not enabled: {}; suspend may lose video memory",
                    disabled.join(" ")
                ),
            )
            .fix(format!("sudo systemctl enable {}", disabled.join(" "))),
        );
    }
    checks
}

/// Value of `Name: value` in /proc/driver/nvidia/params.
fn param<'p>(params: &'p str, name: &str) -> Option<&'p str> {
    params.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == name).then(|| value.trim())
    })
}

fn config(sys: &System) -> Vec<Check> {
    const AREA: &str = "Config";
    let mut checks = Vec::new();

    let entry = sys.home_path(".config/hypr/hyprland.lua");
    match fs::read_to_string(&entry) {
        Ok(text) if text.contains("ghost") => checks.push(Check::new(
            AREA,
            Status::Ok,
            "Ghost Hyprland config deployed",
        )),
        Ok(_) => checks.push(Check::new(
            AREA,
            Status::Warn,
            format!("{} exists but is not Ghost's", entry.display()),
        )),
        Err(_) => checks.push(Check::new(
            AREA,
            Status::Fail,
            "Ghost Hyprland config not deployed (~/.config/hypr/hyprland.lua)",
        )),
    }

    if let Some(host) = sys.hostname() {
        if sys
            .home_path(&format!(".config/hypr/hosts/{host}.lua"))
            .exists()
        {
            checks.push(Check::new(
                AREA,
                Status::Ok,
                format!("Host profile hosts/{host}.lua"),
            ));
        } else {
            checks.push(Check::new(
                AREA,
                Status::Info,
                format!("No hosts/{host}.lua; the default profile is used"),
            ));
        }
    }

    let wallpapers = sys.home_path(".local/share/ghost/wallpapers");
    let has_wallpaper = fs::read_dir(&wallpapers)
        .map(|mut dir| dir.next().is_some())
        .unwrap_or(false);
    if has_wallpaper {
        checks.push(Check::new(AREA, Status::Ok, "Wallpapers present"));
    } else {
        checks.push(Check::new(
            AREA,
            Status::Warn,
            format!("No wallpapers in {}", wallpapers.display()),
        ));
    }
    if !sys.home_path(".local/share/ghost/lockscreen.png").exists() {
        checks.push(Check::new(
            AREA,
            Status::Warn,
            "No lock screen image at ~/.local/share/ghost/lockscreen.png",
        ));
    }
    checks
}

fn session(sys: &System) -> Vec<Check> {
    const AREA: &str = "Session";
    if !sys.in_hyprland() {
        return vec![Check::new(
            AREA,
            Status::Info,
            "Not inside Hyprland; live session checks skipped",
        )];
    }
    match sys.run("hyprctl", &["configerrors"]) {
        Some(out) if out.trim().is_empty() => {
            vec![Check::new(AREA, Status::Ok, "No Hyprland config errors")]
        }
        Some(out) => vec![Check::new(
            AREA,
            Status::Fail,
            format!("Hyprland config errors:\n{}", out.trim()),
        )],
        None => vec![Check::new(AREA, Status::Warn, "Could not run hyprctl")],
    }
}

/// Human-readable report grouped by area.
pub fn render(checks: &[Check], color: bool) -> String {
    let mut out = String::new();
    let mut area = "";
    for check in checks {
        if check.area != area {
            area = check.area;
            let _ = writeln!(out, "{}{area}", if out.is_empty() { "" } else { "\n" });
        }
        let (label, code) = match check.status {
            Status::Ok => ("ok  ", "32"),
            Status::Info => ("info", "34"),
            Status::Warn => ("warn", "33"),
            Status::Fail => ("FAIL", "31"),
        };
        let label = if color {
            format!("\x1b[{code}m{label}\x1b[0m")
        } else {
            label.to_owned()
        };
        let mut lines = check.summary.lines();
        let _ = writeln!(out, "  {label}  {}", lines.next().unwrap_or_default());
        for line in lines {
            let _ = writeln!(out, "        {line}");
        }
        if let Some(fix) = &check.fix {
            let _ = writeln!(out, "        fix: {fix}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::Commands;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    struct FakeCommands(HashMap<String, String>);

    impl Commands for FakeCommands {
        fn stdout(&self, program: &str, args: &[&str]) -> Option<String> {
            self.0
                .get(&format!("{program} {}", args.join(" ")))
                .cloned()
        }
    }

    /// Fixture tree under target/, removed when dropped.
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/test-fixtures")
                .join(name);
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn file(&self, path: &str, contents: &str) -> &Self {
            let full = self.0.join(path.trim_start_matches('/'));
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, contents).unwrap();
            self
        }

        fn card(&self, card: &str, vendor: &str, driver: &str) -> &Self {
            self.file(&format!("sys/class/drm/{card}/device/vendor"), vendor);
            let drivers = self.0.join("sys/bus/pci/drivers").join(driver);
            fs::create_dir_all(&drivers).unwrap();
            std::os::unix::fs::symlink(
                &drivers,
                self.0.join(format!("sys/class/drm/{card}/device/driver")),
            )
            .unwrap();
            self
        }

        fn system<'a>(&self, commands: &'a FakeCommands, in_hyprland: bool) -> System<'a> {
            System::new(
                self.0.clone(),
                self.0.join("home"),
                in_hyprland.then(|| "sig".to_owned()),
                commands,
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn commands(pairs: &[(&str, &str)]) -> FakeCommands {
        FakeCommands(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        )
    }

    fn find<'c>(checks: &'c [Check], text: &str) -> &'c Check {
        checks
            .iter()
            .find(|c| c.summary.contains(text))
            .unwrap_or_else(|| panic!("no check containing {text:?} in {checks:#?}"))
    }

    fn nvidia_fixture(name: &str) -> Fixture {
        let fx = Fixture::new(name);
        fx.card("card0", "0x1002", "amdgpu")
            .card("card1", "0x10de", "nvidia")
            .file("/proc/driver/nvidia/version", "NVRM version: NVIDIA UNIX Open Kernel Module for x86_64  615.71.09  Release Build\n")
            .file("/sys/module/nvidia_drm/parameters/modeset", "Y\n")
            .file("/sys/module/nvidia_drm/parameters/fbdev", "Y\n")
            // As reported by 615 with nvidia-utils' default modprobe options.
            .file("/proc/driver/nvidia/params", "PreserveVideoMemoryAllocations: 2\nUseKernelSuspendNotifiers: 1\n");
        fx
    }

    const LEGACY_PARAMS: &str = "PreserveVideoMemoryAllocations: 1\nUseKernelSuspendNotifiers: 0\n";

    const SERVICES_ON: [(&str, &str); 3] = [
        ("systemctl is-enabled nvidia-suspend.service", "enabled\n"),
        ("systemctl is-enabled nvidia-resume.service", "enabled\n"),
        ("systemctl is-enabled nvidia-hibernate.service", "enabled\n"),
    ];

    #[test]
    fn healthy_nvidia_hybrid_machine() {
        let fx = nvidia_fixture("healthy");
        fx.file("/dev/dri/nvidia-dgpu", "");
        let cmds = commands(&[]);
        let checks = gpu(&fx.system(&cmds, false));
        assert_eq!(find(&checks, "Kernel suspend notifiers").status, Status::Ok);
        assert!(
            checks
                .iter()
                .all(|c| matches!(c.status, Status::Ok | Status::Info)),
            "{checks:#?}"
        );
        assert!(
            find(&checks, "Driver 615.71.09")
                .summary
                .contains("open kernel modules")
        );
    }

    #[test]
    fn hybrid_without_stable_link_warns() {
        let fx = nvidia_fixture("no-link");
        let cmds = commands(&SERVICES_ON);
        let checks = gpu(&fx.system(&cmds, false));
        let check = find(&checks, "no /dev/dri/nvidia-dgpu");
        assert_eq!(check.status, Status::Warn);
        assert!(
            check
                .fix
                .as_deref()
                .unwrap()
                .contains("61-ghost-nvidia-dgpu.rules")
        );
    }

    #[test]
    fn modeset_off_and_legacy_services_disabled() {
        let fx = nvidia_fixture("modeset-off");
        fx.file("/sys/module/nvidia_drm/parameters/modeset", "N\n")
            .file("/proc/driver/nvidia/params", LEGACY_PARAMS);
        let cmds = commands(&[("systemctl is-enabled nvidia-suspend.service", "disabled\n")]);
        let checks = gpu(&fx.system(&cmds, false));
        assert_eq!(find(&checks, "modeset is off").status, Status::Fail);
        let services = find(&checks, "Not enabled");
        assert!(
            services
                .summary
                .contains("nvidia-suspend.service nvidia-resume.service nvidia-hibernate.service")
        );
    }

    #[test]
    fn missing_driver_stops_nvidia_checks() {
        let fx = Fixture::new("no-driver");
        fx.card("card0", "0x10de", "nouveau");
        let cmds = commands(&[]);
        let checks = gpu(&fx.system(&cmds, false));
        assert_eq!(find(&checks, "bound to nouveau").status, Status::Fail);
        assert_eq!(find(&checks, "module not loaded").status, Status::Fail);
        assert!(!checks.iter().any(|c| c.summary.contains("modeset")));
    }

    #[test]
    fn preserve_zero_warns() {
        let fx = nvidia_fixture("preserve-zero");
        fx.file(
            "/proc/driver/nvidia/params",
            "PreserveVideoMemoryAllocations: 0\n",
        );
        let cmds = commands(&SERVICES_ON);
        let checks = gpu(&fx.system(&cmds, false));
        assert_eq!(
            find(&checks, "PreserveVideoMemoryAllocations = 0").status,
            Status::Warn
        );
    }

    #[test]
    fn notifiers_make_services_redundant() {
        let fx = nvidia_fixture("notifiers-and-services");
        let cmds = commands(&SERVICES_ON);
        let checks = gpu(&fx.system(&cmds, false));
        assert_eq!(find(&checks, "Kernel suspend notifiers").status, Status::Ok);
        assert_eq!(find(&checks, "Also enabled").status, Status::Info);
        assert!(!checks.iter().any(|c| c.summary.contains("Not enabled")));
    }

    #[test]
    fn proprietary_module_uses_legacy_suspend_path() {
        let fx = nvidia_fixture("proprietary");
        fx.file(
            "/proc/driver/nvidia/version",
            "NVRM version: NVIDIA UNIX x86_64 Kernel Module  580.95.05  Release Build\n",
        )
        .file("/proc/driver/nvidia/params", LEGACY_PARAMS);
        let cmds = commands(&SERVICES_ON);
        let checks = gpu(&fx.system(&cmds, false));
        assert!(
            find(&checks, "Driver 580.95.05")
                .summary
                .contains("proprietary")
        );
        assert_eq!(
            find(&checks, "Suspend/resume/hibernate services enabled").status,
            Status::Ok
        );
        assert!(!checks.iter().any(|c| c.summary.contains("notifiers")));
    }

    #[test]
    fn gpu_heading_is_not_split_by_nvidia_checks() {
        let fx = nvidia_fixture("ordering");
        let cmds = commands(&[]);
        let checks = gpu(&fx.system(&cmds, false));
        let first_nvidia = checks.iter().position(|c| c.area == "NVIDIA").unwrap();
        assert!(checks[first_nvidia..].iter().all(|c| c.area == "NVIDIA"));
    }

    #[test]
    fn pending_reboot_detected() {
        let fx = Fixture::new("reboot");
        fx.file("/etc/arch-release", "")
            .file("/proc/sys/kernel/osrelease", "7.2.8-1-cachyos-lto\n")
            .file("/usr/lib/modules/7.2.9-1-cachyos-lto/modules.dep", "");
        let cmds = commands(&[]);
        let checks = system(&fx.system(&cmds, false));
        assert_eq!(find(&checks, "Reboot pending").status, Status::Warn);
    }

    #[test]
    fn running_kernel_modules_present() {
        let fx = Fixture::new("kernel-ok");
        fx.file("/etc/arch-release", "")
            .file("/proc/sys/kernel/osrelease", "7.2.9-1-cachyos-lto\n")
            .file("/usr/lib/modules/7.2.9-1-cachyos-lto/modules.dep", "");
        let cmds = commands(&[]);
        let checks = system(&fx.system(&cmds, false));
        assert!(checks.iter().all(|c| c.status == Status::Ok), "{checks:#?}");
    }

    #[test]
    fn missing_packages_listed_with_fix() {
        let fx = Fixture::new("packages");
        let cmds = commands(&[("pacman -Qq", "hyprland\nwaybar\n")]);
        let checks = packages(&fx.system(&cmds, false));
        let check = find(&checks, "packages for apps, core, shell missing");
        assert_eq!(check.status, Status::Fail);
        assert!(check.summary.contains("rofi") && !check.summary.contains(" waybar"));
        assert!(
            !check.summary.contains("sddm"),
            "opt-in components not expected"
        );
        assert!(
            check
                .fix
                .as_deref()
                .unwrap()
                .starts_with("sudo pacman -S --needed ")
        );
        assert_eq!(find(&checks, "wayland-sessions").status, Status::Warn);
    }

    #[test]
    fn packages_follow_deployed_components() {
        let fx = Fixture::new("packages-state");
        fx.file(
            "home/.local/state/ghost/state.json",
            r#"{"version":1,"revision":null,"components":["core","sddm"],"files":{}}"#,
        );
        let cmds = commands(&[("pacman -Qq", "hyprland\n")]);
        let checks = packages(&fx.system(&cmds, false));
        let check = find(&checks, "packages for core, sddm missing");
        assert!(check.summary.contains(" sddm") && !check.summary.contains("waybar"));
    }

    #[test]
    fn config_checks_use_hostname_profile() {
        let fx = Fixture::new("config");
        fx.file("/etc/hostname", "arch-dev\n")
            .file(
                "home/.config/hypr/hyprland.lua",
                "-- ghost — Hyprland entry point\n",
            )
            .file(
                "home/.config/hypr/hosts/arch-dev.lua",
                "return { monitors = {} }\n",
            )
            .file("home/.local/share/ghost/wallpapers/a.png", "");
        let cmds = commands(&[]);
        let checks = config(&fx.system(&cmds, false));
        assert_eq!(
            find(&checks, "Host profile hosts/arch-dev.lua").status,
            Status::Ok
        );
        assert_eq!(find(&checks, "lock screen").status, Status::Warn);
        assert_eq!(
            find(&checks, "Ghost Hyprland config deployed").status,
            Status::Ok
        );
    }

    #[test]
    fn live_session_reports_config_errors() {
        let fx = Fixture::new("session");
        let cmds = commands(&[("hyprctl configerrors", "ghost/binds.lua:3: bad bind\n")]);
        let checks = session(&fx.system(&cmds, true));
        assert_eq!(checks[0].status, Status::Fail);
        assert!(checks[0].summary.contains("bad bind"));
        let outside = session(&fx.system(&cmds, false));
        assert_eq!(outside[0].status, Status::Info);
    }

    #[test]
    fn render_groups_by_area() {
        let checks = vec![
            Check::new("System", Status::Ok, "Arch Linux"),
            Check::new("GPU", Status::Warn, "two\nlines").fix("do it"),
        ];
        assert_eq!(
            render(&checks, false),
            "System\n  ok    Arch Linux\n\nGPU\n  warn  two\n        lines\n        fix: do it\n"
        );
    }
}
