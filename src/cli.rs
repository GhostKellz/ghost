use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use crate::deploy::{self, Kind, Ops, Paths, Plan};
use crate::manifest::{self, Manifest};
use crate::state::{self, State};
use crate::{doctor, system};

/// Themeable Hyprland desktop installer for Arch Linux.
#[derive(Debug, Parser)]
#[command(name = "ghost", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Deploy Ghost's configuration; re-run to update. Shows the plan and asks first.
    Install {
        /// Print every file action without changing anything.
        #[arg(long)]
        dry_run: bool,
        /// Proceed without asking.
        #[arg(long, short)]
        yes: bool,
        /// Repository checkout to deploy from.
        #[arg(long, default_value = ".")]
        source: PathBuf,
        /// Also install an opt-in component (repeatable).
        #[arg(long, value_name = "COMPONENT")]
        with: Vec<String>,
        /// Skip a default component (repeatable).
        #[arg(long, value_name = "COMPONENT")]
        without: Vec<String>,
        /// Theme to deploy.
        #[arg(long)]
        theme: Option<String>,
        /// Host profile under config/hypr/hosts/ (defaults to the hostname).
        #[arg(long)]
        host: Option<String>,
        /// Enable translucent surfaces at the default opacity.
        #[arg(long)]
        translucent: bool,
        /// Surface opacity in percent; implies --translucent.
        #[arg(long, value_parser = clap::value_parser!(u8).range(50..=100))]
        opacity: Option<u8>,
    },
    /// Re-render and apply theme files without reinstalling packages.
    Apply {
        #[arg(long)]
        theme: Option<String>,
    },
    /// Undo the most recent install; repeat to step further back.
    Restore {
        /// List restorable backups instead.
        #[arg(long)]
        list: bool,
        /// Show what would be restored without changing anything.
        #[arg(long)]
        dry_run: bool,
        /// Proceed without asking.
        #[arg(long, short)]
        yes: bool,
    },
    /// Check packages, GPU and driver setup, deployed config, and the live session.
    Doctor,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Doctor => doctor(),
            Command::Install {
                dry_run,
                yes,
                source,
                with,
                without,
                theme,
                host,
                translucent,
                opacity,
            } => {
                if theme.is_some() || host.is_some() || translucent || opacity.is_some() {
                    bail!("--theme, --host, --translucent and --opacity are not implemented yet");
                }
                install(&source, &with, &without, dry_run, yes)
            }
            Command::Restore { list, dry_run, yes } => restore(list, dry_run, yes),
            // Fail loudly so nothing mistakes an unfinished command for a working one.
            Command::Apply { .. } => bail!("`ghost apply` is not implemented yet"),
        }
    }
}

fn doctor() -> anyhow::Result<()> {
    let commands = system::HostCommands;
    let sys = system::System::host(&commands)?;
    let checks = doctor::run(&sys);
    print!("{}", doctor::render(&checks, io::stdout().is_terminal()));
    let failures = checks
        .iter()
        .filter(|c| c.status == doctor::Status::Fail)
        .count();
    if failures > 0 {
        bail!("{failures} check(s) failed");
    }
    Ok(())
}

struct User {
    home: PathBuf,
    paths: Paths,
}

/// Ghost manages one user's files, so it must run as that user, not root.
fn user() -> anyhow::Result<User> {
    let status =
        std::fs::read_to_string("/proc/self/status").context("reading /proc/self/status")?;
    let euid = status
        .lines()
        .find_map(|l| l.strip_prefix("Uid:"))
        .and_then(|ids| ids.split_whitespace().nth(1));
    if euid == Some("0") {
        bail!("run ghost as your desktop user, not root; it asks for sudo when a step needs it");
    }
    let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
    Ok(User {
        paths: Paths::under(&state::dir(&home, std::env::var_os("XDG_STATE_HOME"))),
        home,
    })
}

fn install(
    source: &Path,
    with: &[String],
    without: &[String],
    dry_run: bool,
    yes: bool,
) -> anyhow::Result<()> {
    let user = user()?;
    let manifest = Manifest::load(source)?;
    let mut state = State::load(&user.paths.state)?;

    // Components installed earlier stay installed unless excluded, so a plain
    // re-run never removes an opt-in component such as sddm.
    let mut with = with.to_vec();
    with.extend(
        state
            .components
            .iter()
            .filter(|c| manifest.components.contains_key(*c) && !without.contains(c))
            .cloned(),
    );
    with.sort();
    with.dedup();
    let chosen = manifest.select(&with, without)?;

    let entries = manifest.entries(source, &user.home, &chosen)?;
    let plan = deploy::plan(&entries, &state)?;
    let ops = Ops::new(&user.home, true, root_destinations(Some(&manifest)));
    let missing = missing_packages(&manifest.packages(&chosen))?;
    let hooks = hooks_for(&manifest, &plan, &user.home);

    println!("Components:");
    for name in &chosen {
        println!("  {name}: {}", manifest.components[name].description);
    }
    println!();
    if missing.is_empty() {
        println!("Packages: all installed");
    } else {
        println!(
            "Packages to install (sudo pacman -S --needed):\n  {}",
            missing.join(" ")
        );
    }
    println!();
    print!("{}", plan.render(&user.home));
    let root_writes = plan
        .actions
        .iter()
        .any(|a| a.kind.changes_disk() && ops.needs_root(&a.dest));
    if root_writes {
        println!("Paths outside your home directory are written with sudo.");
    }
    for hook in &hooks {
        println!("Then: sudo {}", hook.join(" "));
    }

    if dry_run {
        println!("\nDry run: nothing changed.");
        return Ok(());
    }
    if missing.is_empty() && !plan.has_changes() && state.components == chosen {
        println!("\nNothing to do.");
        return Ok(());
    }
    if !missing.is_empty() {
        // Installing against a database newer than the installed system is a
        // partial upgrade. Leave the full upgrade to the user.
        let pending = pending_updates()?;
        if !pending.is_empty() {
            bail!(
                "{} package update(s) pending; run `sudo pacman -Syu` and reboot if needed, then re-run ghost install",
                pending.len()
            );
        }
    }
    if !yes && !confirm("\nApply these changes?")? {
        bail!("cancelled");
    }

    if !missing.is_empty() {
        let mut args = vec!["pacman", "-S", "--needed"];
        if yes {
            args.push("--noconfirm");
        }
        args.push("--");
        args.extend(missing.iter().map(String::as_str));
        run_sudo(&args)?;
    }
    let backup = deploy::execute(
        &plan,
        &mut state,
        &user.paths,
        &ops,
        revision(source),
        chosen,
    )?;
    for hook in &hooks {
        let args: Vec<&str> = hook.iter().map(String::as_str).collect();
        run_sudo(&args)?;
    }
    println!();
    if let Some(id) = backup {
        println!("Done. Backup {id}; undo with `ghost restore`.");
    } else {
        println!("Done.");
    }
    let conflicts: Vec<String> = plan
        .actions
        .iter()
        .filter(|a| a.kind == Kind::Conflict)
        .map(|a| deploy::display(&deploy::ghost_new(&a.dest), &user.home))
        .collect();
    if !conflicts.is_empty() {
        println!(
            "Review and merge Ghost's new versions:\n  {}",
            conflicts.join("\n  ")
        );
    }
    Ok(())
}

fn restore(list: bool, dry_run: bool, yes: bool) -> anyhow::Result<()> {
    let user = user()?;
    if list {
        let backups = deploy::list_backups(&user.paths)?;
        if backups.is_empty() {
            println!("No backups.");
        }
        for b in backups {
            println!(
                "{}  {} created, {} saved  ({})",
                b.id,
                b.created.len(),
                b.saved.len(),
                b.revision.as_deref().unwrap_or("unknown revision")
            );
        }
        return Ok(());
    }

    let ops = Ops::new(&user.home, true, root_destinations(None));
    let Some((id, preview)) = deploy::restore_latest(&user.paths, &ops, true)? else {
        println!("Nothing to restore.");
        return Ok(());
    };
    println!("Restore {id}:");
    for (label, paths) in [
        ("put back", &preview.restored),
        ("remove", &preview.removed),
        ("keep (changed since install)", &preview.kept),
    ] {
        for path in paths {
            println!("  {label}: {}", deploy::display(path, &user.home));
        }
    }
    if dry_run {
        println!("\nDry run: nothing changed.");
        return Ok(());
    }
    if !yes && !confirm("\nRestore?")? {
        bail!("cancelled");
    }
    deploy::restore_latest(&user.paths, &ops, false)?;
    // A restored udev rule or theme needs the same reload as an installed one.
    if let Ok(manifest) = Manifest::parse(manifest::EMBEDDED) {
        let touched: Vec<&Path> = preview
            .restored
            .iter()
            .chain(&preview.removed)
            .map(PathBuf::as_path)
            .collect();
        for hook in hooks_for_paths(&manifest, &touched, &user.home) {
            let args: Vec<&str> = hook.iter().map(String::as_str).collect();
            run_sudo(&args)?;
        }
    }
    println!("Restored {id}.");
    Ok(())
}

/// System paths sudo may touch: root destinations from the embedded manifest,
/// plus the checkout's when it differs (files from either may need removing).
fn root_destinations(source: Option<&Manifest>) -> Vec<PathBuf> {
    let mut dests = Manifest::parse(manifest::EMBEDDED)
        .map(|m| m.root_destinations())
        .unwrap_or_default();
    if let Some(m) = source {
        dests.extend(m.root_destinations());
    }
    dests.sort();
    dests.dedup();
    dests
}

/// Hooks of root components whose files the plan changes, in manifest order.
fn hooks_for(manifest: &Manifest, plan: &Plan, home: &Path) -> Vec<Vec<String>> {
    let changed: Vec<&Path> = plan
        .actions
        .iter()
        .filter(|a| a.kind.changes_disk())
        .map(|a| a.dest.as_path())
        .collect();
    hooks_for_paths(manifest, &changed, home)
}

fn hooks_for_paths(manifest: &Manifest, paths: &[&Path], home: &Path) -> Vec<Vec<String>> {
    let mut hooks = Vec::new();
    for component in manifest.components.values().filter(|c| c.root) {
        let touched = component.files.iter().any(|mapping| {
            let dest = match mapping.dest.strip_prefix("~/") {
                Some(rest) => home.join(rest),
                None => PathBuf::from(&mapping.dest),
            };
            paths.iter().any(|p| p.starts_with(&dest))
        });
        if touched {
            hooks.extend(component.hooks.iter().cloned());
        }
    }
    hooks
}

fn missing_packages(wanted: &[&str]) -> anyhow::Result<Vec<String>> {
    let out = std::process::Command::new("pacman")
        .arg("-Qq")
        .output()
        .context("running pacman -Qq")?;
    let installed = String::from_utf8_lossy(&out.stdout);
    let installed: Vec<&str> = installed.lines().map(str::trim).collect();
    Ok(wanted
        .iter()
        .filter(|p| !installed.contains(p))
        .map(|p| (*p).to_owned())
        .collect())
}

/// Installed packages older than the local sync database (`pacman -Qu`).
fn pending_updates() -> anyhow::Result<Vec<String>> {
    let out = std::process::Command::new("pacman")
        .arg("-Qu")
        .output()
        .context("running pacman -Qu")?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect())
}

fn run_sudo(args: &[&str]) -> anyhow::Result<()> {
    let status = std::process::Command::new("sudo")
        .args(args)
        .status()
        .context("running sudo")?;
    if !status.success() {
        bail!("sudo {} failed ({status})", args.join(" "));
    }
    Ok(())
}

fn confirm(question: &str) -> anyhow::Result<bool> {
    if !io::stdin().is_terminal() {
        bail!("not a terminal; pass --yes to proceed without confirmation");
    }
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

fn revision(source: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(source)
        .args(["describe", "--always", "--dirty"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn opacity_out_of_range_is_rejected() {
        let result = Cli::try_parse_from(["ghost", "install", "--opacity", "20"]);
        assert!(result.is_err());
    }

    #[test]
    fn hooks_run_only_for_touched_root_components() {
        let manifest = Manifest::parse(manifest::EMBEDDED).unwrap();
        let home = Path::new("/home/u");
        let udev = Path::new("/etc/udev/rules.d/61-ghost-nvidia-dgpu.rules");
        let hooks = hooks_for_paths(&manifest, &[udev], home);
        assert_eq!(hooks.len(), 3);
        assert_eq!(hooks[2][..2], ["udevadm", "settle"]);
        assert_eq!(hooks[0], ["udevadm", "control", "--reload"]);
        let user_file = Path::new("/home/u/.config/hypr/hyprland.lua");
        assert!(hooks_for_paths(&manifest, &[user_file], home).is_empty());
        let theme = Path::new("/usr/share/sddm/themes/ghost/Main.qml");
        assert!(hooks_for_paths(&manifest, &[theme], home).is_empty());
    }

    #[test]
    fn unimplemented_commands_fail() {
        let cli = Cli::try_parse_from(["ghost", "apply"]).unwrap();
        assert!(cli.run().is_err());
        let cli = Cli::try_parse_from(["ghost", "install", "--theme", "storm"]).unwrap();
        assert!(cli.run().is_err());
    }
}
