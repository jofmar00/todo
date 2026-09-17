mod args;
mod commands;
mod task;

use args::Args;
use clap::Parser;
use commands::Command;
use std::error::Error;

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
    let mut ls_tags = Vec::new();
    let mut show_list = true;
    match args.cmd {
        Command::Add { description, tag } => tasks.add(description, tag),
        Command::Done { ids } => tasks.mark_done(ids),
        Command::Undone { ids } => tasks.mark_undone(ids),
        Command::Rm { ids } => tasks.remove(ids),
        Command::Rmtag { tags } => tasks.remove_tag(tags),
        Command::Tag { id, tag } => tasks.tag(id, tag),
        Command::Use { tag } => tasks.use_tag(tag),
        Command::Order => tasks.order_completed(),
        Command::Lstags => { tasks.list_tags(); show_list = false; },
        // TODO: Hacer ls para los tags
        Command::Ls { tags } => ls_tags = tags,
    };

    if show_list { tasks.list(ls_tags); }
    
    tasks.save_all()?;
    Ok(())
}
