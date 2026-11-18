use crate::{
    manager::{TaskManager, TaskManagerResult},
    tag::Tag,
    task::TaskID,
};
use clap::{Parser, Subcommand};
use std::{fmt::Debug, num::ParseIntError, str::FromStr};
use thiserror::Error;

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
        tag: Option<Tag>,
    },
    /// List all tasks
    #[command(alias = "l")]
    Ls {
        /// Tag
        tags: Vec<Tag>,
    },
    /// Mark a task as done
    Done {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<TaskIdParam>,
    },
    /// Unmark a task as done
    Undone {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<TaskIdParam>,
    },
    /// Remove a task
    Rm {
        /// Task ID
        #[arg(required = true, num_args = 1..)]
        ids: Vec<TaskIdParam>,
    },
    /// Order completed tasks to be first in the list
    Order,

    /// Assign a tag to a task
    Tag {
        /// Task ID
        id: TaskID,

        /// Tag
        tag: Tag,
    },
    /// List all tags
    Lstags,
    /// Use a specific tag in listing and creation of new tags
    Use {
        /// Tag
        tag: Option<Tag>,
    },
    /// Rename tag
    Rename { original: Tag, new: Tag },
    /// Remove all the tasks assign to a tag
    Rmtag {
        /// Tag
        tags: Vec<Tag>,
    },
    /// Show progress of actual tag
    Progress,
}

impl Command {
    pub fn execute(self, tasks: &mut TaskManager) -> TaskManagerResult {
        let mut ls_tags = Vec::new();
        let mut show_list = true;
        match self {
            Command::Add { description, tag } => tasks.add(description, tag),
            Command::Ls { tags } => ls_tags = tags,
            Command::Done { ids } => tasks.mark_done(TaskIdParam::to_id_vec(ids))?,
            Command::Undone { ids } => tasks.mark_undone(TaskIdParam::to_id_vec(ids))?,
            Command::Rm { ids } => tasks.remove(TaskIdParam::to_id_vec(ids))?,
            Command::Order => tasks.order_completed(),
            Command::Tag { id, tag } => tasks.tag(id, tag)?,
            Command::Lstags => {
                tasks.list_tags();
                show_list = false;
            }
            Command::Use { tag } => tasks.use_tag(tag)?,
            Command::Rename { original, new } => tasks.rename_tag(original, new),
            Command::Rmtag { tags } => tasks.remove_tag(tags),
            Command::Progress => {
                tasks.show_progress()?;
                show_list = false;
            }
        }
        if show_list {
            tasks.list(ls_tags)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum TaskIdParam {
    One(TaskID),
    Range { start: TaskID, end: TaskID },
}

#[derive(Error, Debug)]
pub enum TaskIdParamError {
    #[error("invalid value {value}: {source}")]
    InvalidValue {
        value: String,
        source: ParseIntError,
    },

    #[error("invalid range: {0}")]
    InvalidRange(String),
}

impl FromStr for TaskIdParam {
    type Err = TaskIdParamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_one = |v: &str| {
            v.parse::<TaskID>()
                .map_err(|e| TaskIdParamError::InvalidValue {
                    value: v.to_string(),
                    source: e,
                })
        };

        match s.split_once('-') {
            Some((start, end)) => {
                let start = parse_one(start)?;
                let end = parse_one(end)?;
                if end < start {
                    return Err(TaskIdParamError::InvalidRange(s.to_string()));
                }
                Ok(Self::Range { start, end })
            }
            None => Ok(Self::One(parse_one(s)?)),
        }
    }
}

impl TaskIdParam {
    fn to_id_vec(params: Vec<TaskIdParam>) -> Vec<TaskID> {
        let mut ids = Vec::new();

        for param in params {
            match param {
                Self::One(id) => ids.push(id),
                Self::Range { start, end } => ids.extend(start..=end),
            }
        }

        ids
    }
}
