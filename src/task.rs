use crate::tag::Tag;
#[cfg(test)]
use bon::Builder;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Builder))]
pub struct Task {
    #[serde(skip)]
    #[cfg_attr(test, builder(default))]
    pub id: u32,

    #[cfg_attr(test, builder(default))]
    pub description: TaskDescription,

    #[cfg_attr(test, builder(default))]
    pub completed: bool,

    pub tag: Option<Tag>,
}
pub type TaskID = u32;
pub type TaskDescription = String;

impl Task {
    pub fn new(id: u32, description: String, tag: Option<Tag>) -> Self {
        Self {
            id,
            description,
            tag,
            completed: false,
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.completed {
            "[x]".bright_green().bold()
        } else {
            "[ ]".bold()
        };

        let tag = match self.tag {
            Some(ref tag) => format!("({tag})"),
            None => "()".into(),
        };

        let description = if self.completed {
            self.description.strikethrough().dimmed()
        } else {
            self.description.normal()
        };

        write!(f, "{status}{tag} {description}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_inits_task_as_uncompleted() {
        let task = Task::new(1, "Write tests".into(), None);

        assert_eq!(task.id, 1);
        assert_eq!(task.description, "Write tests");
        assert!(!task.completed);
        assert!(task.tag.is_none());
    }

    #[test]
    fn display_shows_task_status_and_content() {
        let pending = Task::builder().description("Write tests".into()).build();
        let completed = Task::builder()
            .completed(true)
            .description("Ship changes".into())
            .tag("work".into())
            .build();

        let pending_message = pending.to_string();
        let completed_message = completed.to_string();

        assert!(pending_message.contains("[ ]"));
        assert!(pending_message.contains("()"));
        assert!(pending_message.contains("Write tests"));

        println!("{completed_message}");
        assert!(completed_message.contains("[x]"));
        assert!(completed_message.contains("#work"));
        assert!(completed_message.contains("Ship changes"));
    }
}
