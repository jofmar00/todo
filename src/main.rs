mod commands;
mod task;

use commands::{Args,Command};
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
    let mut ls_tags = Vec::new();
    let mut show_list = true;
    match args.cmd {
        Command::Add { description, tag } => tasks.add(description, tag),
        Command::Ls { tags } => ls_tags = tags,
        Command::Done { ids } => tasks.mark_done(ids),
        Command::Undone { ids } => tasks.mark_undone(ids),
        Command::Rm { ids } => tasks.remove(ids),
        Command::Order => tasks.order_completed(),
        Command::Tag { id, tag } => tasks.tag(id, tag),
        Command::Lstags => { tasks.list_tags(); show_list = false; },
        Command::Use { tag } => tasks.use_tag(tag),
        Command::Rename { original, new } => tasks.rename_tag(original, new),
        Command::Rmtag { tags } => tasks.remove_tag(tags),
    };

    if show_list { tasks.list(ls_tags); }
    
    tasks.save_all()?;
    Ok(())
}
