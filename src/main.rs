mod commands;
mod consts;
mod tag;
mod task;

pub mod manager;
use crate::manager::TaskManager;
use colored::Colorize;
use crate::commands::Args;
use std::error::Error;
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "Error:".red());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut manager = TaskManager::build()?;
    match args.cmd {
        Some(cmd) => cmd.execute(&mut manager)?,
        None => manager.list(Vec::new())?,
    };
    manager.save_tasks()?;
    Ok(())
}
