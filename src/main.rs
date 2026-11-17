mod commands;
mod consts;
mod tag;
mod task;

pub mod manager;
use crate::{commands::Args, manager::TaskManager};
use clap::Parser;
use colored::Colorize;
use std::error::Error;

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
