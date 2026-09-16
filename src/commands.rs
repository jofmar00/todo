use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Add a task
    Add {
        /// Task message description
        description: String,
        /// Tag to assign to the task (defaults to the selected tag, if any)
        tag: Option<String>,
    },
    /// Mark a task as done
    Done {
        /// Task ID
        id: u32,
    },
    /// Unmark a task as done
    Undone {
        /// Task ID
        id: u32,
    },
    /// Remove a task
    Rm {
        /// Task ID
        id: u32,
    },
    /// Remove all the tasks assign to a tag
    Rmtag {
        /// Tag
        tag: String
    },
    /// Assign a tag to a task
    Tag {
        /// Task ID
        id: u32,

        /// Tag
        tag: String
    },
    /// Select the tag to work with, filtering the task list and tagging new tasks by default
    Use {
        /// Tag
        tag: Option<String>
   },
    /// Order completed tasks to be first in the list
    Order,
    /// List all tasks
    Ls {
        /// Tag
        tag: Option<String>
    }
}
