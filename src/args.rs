use clap::Parser;

use crate::commands::Command;

/// A simple command-line to-do list manager. Made by @jofmar00 in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    pub cmd: Command,
}
