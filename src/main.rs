mod commands;
mod task;

use crate::task::TaskManager;
use clap::Parser;
use colored::Colorize;
use commands::Args;
use std::error::Error;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "Error:".red());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut tasks = TaskManager::build()?;
    match args.cmd {
        Some(cmd) => cmd.execute(&mut tasks),
        None => tasks.list(Vec::new()),
    }
    tasks.save_all()?;
    Ok(())
}
