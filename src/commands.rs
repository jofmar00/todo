use clap::{Parser, Subcommand};

use crate::task::TaskManager;

/// A simple command-line to-do list manager. Made by @jofmar00 in Rust.
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a task
    #[command(alias = "a")]
    Add {
        /// Task message description
        description: String,
        /// Tag to assign to the task (defaults to the selected tag, if any)
        tag: Option<String>,
    },
    /// List all tasks
    #[command(alias = "l")]
    Ls {
        /// Tag
        tags: Vec<String>
    },
    /// Mark a task as done
    Done {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<u32>,
    },
    /// Unmark a task as done
    Undone {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<u32>,
    },
    /// Remove a task
    Rm {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<u32>,
    },
    /// Order completed tasks to be first in the list
    Order,

    /// Assign a tag to a task
    Tag {
        /// Task ID
        id: u32,

        /// Tag
        tag: String
    },
    /// List all tags
    Lstags,
    /// Use a specific tag in listing and creation of new tags
    Use {
        /// Tag
        tag: Option<String>
   },
    /// Rename tag
    Rename {
        original: String,
        new: String,
    },
    /// Remove all the tasks assign to a tag
    Rmtag {
        /// Tag
        #[arg(required = true, num_args = 1..)]
        tags: Vec<String>,
    },
    /// Show progress of actual tag
    Progress,
}

impl Command {
    pub fn execute(self, tasks: &mut TaskManager) {
        let mut ls_tags = Vec::new();
        let mut show_list = true;
        match self {
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
            Command::Progress => { tasks.show_progress(); show_list = false; },
        }
        if show_list { tasks.list(ls_tags); }
    }
}
