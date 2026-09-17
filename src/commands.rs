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
    /// Remove all the tasks assign to a tag
    Rmtag {
        /// Tag
        #[arg(required = true, num_args = 1..)]
        tags: Vec<String>,
    },
    /// Assign a tag to a task
    Tag {
        /// Task ID
        id: u32,

        /// Tag
        tag: String
    },
    /// Use a specific tag in listing and creation of new tags
    Use {
        /// Tag
        tag: Option<String>
   },
    /// Order completed tasks to be first in the list
    Order,
    /// List all tags
    Lstags,
    /// List all tasks
    Ls {
        /// Tag
        tags: Vec<String>
    },
}
