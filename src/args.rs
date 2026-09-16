use clap::Parser;

use crate::commands::Command;

/// A simple command-line to-do list manager.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    pub cmd: Command,
}
