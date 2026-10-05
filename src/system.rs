use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Read-only view of the machine. File paths are resolved under `root` and
/// commands go through `Commands`, so checks can run against fixtures in tests.
pub struct System<'a> {
    root: PathBuf,
    home: PathBuf,
    /// Ghost's state directory (see `state::dir`).
    pub state_dir: PathBuf,
    hyprland_instance: Option<String>,
    commands: &'a dyn Commands,
}

pub trait Commands {
    /// Stdout of the command, whatever its exit status; `None` if it could not run.
    fn stdout(&self, program: &str, args: &[&str]) -> Option<String>;
}

pub struct HostCommands;

impl Commands for HostCommands {
    fn stdout(&self, program: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(program).args(args).output().ok()?;
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// A DRM card (`/sys/class/drm/cardN`), i.e. one GPU.
#[derive(Debug, Clone, PartialEq)]
pub struct Gpu {
    pub card: String,
    pub vendor: String,
    pub driver: Option<String>,
}

pub const VENDOR_NVIDIA: &str = "0x10de";

impl Gpu {
    pub fn is_nvidia(&self) -> bool {
        self.vendor == VENDOR_NVIDIA
    }
}

impl<'a> System<'a> {
    pub fn new(
        root: PathBuf,
        home: PathBuf,
        hyprland_instance: Option<String>,
        commands: &'a dyn Commands,
    ) -> Self {
        Self {
            state_dir: crate::state::dir(&home, None),
            root,
            home,
            hyprland_instance,
            commands,
        }
    }

    pub fn host(commands: &'a dyn Commands) -> anyhow::Result<Self> {
        let home = std::env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
        let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .ok()
            .filter(|s| !s.is_empty());
        let mut sys = Self::new(PathBuf::from("/"), PathBuf::from(home), instance, commands);
        sys.state_dir = crate::state::dir(&sys.home, std::env::var_os("XDG_STATE_HOME"));
        Ok(sys)
    }

    /// Absolute system path resolved under the probe root.
    pub fn path(&self, absolute: &str) -> PathBuf {
        self.root.join(absolute.trim_start_matches('/'))
    }

    /// Path relative to the user's home directory.
    pub fn home_path(&self, relative: &str) -> PathBuf {
        self.home.join(relative)
    }

    pub fn exists(&self, absolute: &str) -> bool {
        self.path(absolute).exists()
    }

    /// Trimmed file contents, or `None` if unreadable.
    pub fn read(&self, absolute: &str) -> Option<String> {
        read_trimmed(&self.path(absolute))
    }

    pub fn run(&self, program: &str, args: &[&str]) -> Option<String> {
        self.commands.stdout(program, args)
    }

    pub fn in_hyprland(&self) -> bool {
        self.hyprland_instance.is_some()
    }

    pub fn hostname(&self) -> Option<String> {
        self.read("/etc/hostname").filter(|name| !name.is_empty())
    }

    pub fn gpus(&self) -> Vec<Gpu> {
        let Ok(entries) = fs::read_dir(self.path("/sys/class/drm")) else {
            return Vec::new();
        };
        let mut gpus: Vec<Gpu> = entries
            .flatten()
            .filter_map(|entry| {
                let card = entry.file_name().to_string_lossy().into_owned();
                let is_card = card
                    .strip_prefix("card")?
                    .chars()
                    .all(|c| c.is_ascii_digit());
                if !is_card || card == "card" {
                    return None;
                }
                let device = entry.path().join("device");
                let vendor = read_trimmed(&device.join("vendor"))?;
                let driver = fs::read_link(device.join("driver"))
                    .ok()
                    .and_then(|target| {
                        target
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                    });
                Some(Gpu {
                    card,
                    vendor,
                    driver,
                })
            })
            .collect();
        gpus.sort_by(|a, b| a.card.cmp(&b.card));
        gpus
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_owned())
}
