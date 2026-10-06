mod cli;
mod deploy;
mod doctor;
mod manifest;
mod sources;
mod state;
mod system;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    cli::Cli::parse().run()
}
