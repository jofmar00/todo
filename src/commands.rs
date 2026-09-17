use clap::{Parser, Subcommand};

/// A simple command-line to-do list manager. Made by @jofmar00 in Rust.
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a task
    Add {
        /// Task message description
        description: String,
        /// Tag to assign to the task (defaults to the selected tag, if any)
        tag: Option<String>,
    },
    /// List all tasks
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
}
