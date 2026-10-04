use anyhow::bail;
use clap::{Parser, Subcommand};

/// Themeable Hyprland desktop installer for Arch Linux.
#[derive(Debug, Parser)]
#[command(name = "ghost", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Install packages and deploy the ghost desktop.
    Install {
        /// Print every package and file action without changing anything.
        #[arg(long)]
        dry_run: bool,
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
    /// Restore the configuration backed up by the last install.
    Restore,
    /// Check packages, plugins, portals, and GPU setup.
    Doctor,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let name = match self.command {
            Command::Install { .. } => "install",
            Command::Apply { .. } => "apply",
            Command::Restore => "restore",
            Command::Doctor => "doctor",
        };
        // Fail loudly so nothing mistakes the skeleton for a working installer.
        bail!("`ghost {name}` is not implemented yet")
    }
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
    fn unimplemented_commands_fail() {
        let cli = Cli::try_parse_from(["ghost", "doctor"]).unwrap();
        assert!(cli.run().is_err());
    }
}
