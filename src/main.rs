mod args;
mod commands;
mod task;

use args::Args;
use clap::Parser;
use commands::Command;
use std::error::Error;

use crate::task::TaskManager;

pub const TODO_PATH: &str = ".todo_list";

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
        Command::Add { description, tag } => tasks.add(description, tag),
        Command::Done { id } => tasks.mark_done(id),
        Command::Undone { id } => tasks.mark_undone(id),
        Command::Rm { id } => tasks.remove(id),
        Command::Tag { id, tag } => tasks.tag(id, tag),
        Command::Select { tag } => tasks.select(tag),
        Command::Order => tasks.order_completed(),
        Command::Ls => (),
    };
    tasks.list();
    tasks.save_all()?;
    Ok(())
}
