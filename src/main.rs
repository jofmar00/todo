mod commands;
mod task;

use commands::Args;
use std::error::Error;
use clap::Parser;

use crate::task::TaskManager;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
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
