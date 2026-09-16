use std::{error::Error, fmt::Display, fs::OpenOptions, io::Write, path::Path};

use serde::{Deserialize, Serialize};

use crate::TODO_PATH;

const DEFAULT_TAG: &str = "default";

// TODO: Igual hay alguna manera mejor de hacer esto
fn save_err(e: impl Display) -> String {
    format!("No se pudo guardar el fichero {TODO_PATH}: {e}")
}

#[derive(Serialize, Deserialize)]
struct Task {
    #[serde(skip)]
    id: u32,
    description: String,
    completed: bool,
    tag: Option<String>,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.completed { "x" } else { " " };
        write!(f, "{} [{status}] - {}", self.id, self.description)?;
        if let Some(tag) = &self.tag {
            write!(f, " (#{tag})")?;
        }
        Ok(())
    }
}

impl Task {
    fn new(id: u32, description: String, tag: Option<String>) -> Self {
        Self {
            id,
            description,
            tag,
            completed: false,
        }
    }
}

pub struct TaskManager {
    tasks: Vec<Task>,
    selected_tag: Option<String>
}

impl TaskManager {
    pub fn build() -> Result<Self, Box<dyn Error>> {
        let mut tasks = Vec::new();
        if !Path::new(TODO_PATH).exists() {
            return Ok(Self { tasks, selected_tag: None });
        }

        let contents = std::fs::read_to_string(TODO_PATH)
            .map_err(|e| format!("No se pudo leer el fichero {TODO_PATH}: {e}"))?;

        // Obtain tag
        let mut lines = contents.lines();
        let selected_tag = lines.next().filter(|l| *l != DEFAULT_TAG).map(String::from);
        let rest = lines.collect::<Vec<_>>().join("\n");

        // Load tasks in memory
        let mut csv_reader = csv::ReaderBuilder::new().from_reader(rest.as_bytes());
        for (i, result) in csv_reader.deserialize().enumerate() {
            let mut task: Task = result
                .map_err(|e| format!("No se pudo parsear el CSV de {TODO_PATH}: {e}"))?;
            task.id = i as u32 + 1;
            tasks.push(task);
        }
        Ok(Self { tasks, selected_tag })
    }

    fn next_id(&self) -> u32 {
        self.tasks.len() as u32 + 1
    }

    fn recalculate_task_ids(&mut self) {
        for (i, task) in self.tasks.iter_mut().enumerate() {
            task.id = i as u32 + 1;
        }
    }

    fn progress_by_tag(&self) -> f32 {
        let (done, total) = self.tasks.iter()
            .filter(|t| self.selected_tag.is_none() || t.tag == self.selected_tag)
            .fold((0u32, 0u32), |(done, total), t| (done + t.completed as u32, total + 1));

        if total == 0 { 0.0 } else { done as f32 / total as f32 }
    }

    pub fn add(&mut self, description: String, tag: Option<String>) {
        let tag = tag.or_else(|| self.selected_tag.clone());
        let task = Task::new(self.next_id(), description, tag);
        self.tasks.push(task);
    }

    pub fn list(&self) {
        self.tasks.iter()
            .filter(|t| self.selected_tag.is_none() || t.tag == self.selected_tag )
            .for_each(|t| println!("{t}"));
       
        println!(
            "You have completed {}% of {} tasks",
            self.progress_by_tag() * 100.0,
            self.selected_tag.as_deref().unwrap_or(DEFAULT_TAG)
        );
    }

    pub fn select(&mut self, tag: String) {
        println!("[INFO] Selected tag {tag}\n");
        self.selected_tag = if tag == DEFAULT_TAG { None } else { Some(tag) };
    }

    pub fn mark_done(&mut self, id: u32) {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => task.completed = true,
            None => eprintln!("Task not found"),
        }
    }

    pub fn mark_undone(&mut self, id: u32) {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => task.completed = false,
            None => eprintln!("Task not found"),
        }
    }

    pub fn tag(&mut self, id: u32, tag: String) {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => task.tag = Some(tag),
            None => eprintln!("Task not found"),
        }
    }

    pub fn save_all(&self) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(TODO_PATH)
            .map_err(save_err)?;
        writeln!(file, "{}", self.selected_tag.as_deref().unwrap_or(DEFAULT_TAG)).map_err(save_err)?;

        let mut csv_wtr = csv::WriterBuilder::new().from_writer(file);
        for task in &self.tasks {
            csv_wtr.serialize(task).map_err(save_err)?;
        }
        csv_wtr.flush().map_err(save_err)?;
        Ok(())
    }

    pub fn remove(&mut self, id: u32) {
        match self.tasks.iter().find(|t| t.id == id) {
            Some(delete_task) => {
                println!("[INFO] Deleted task: {} \n", delete_task.description);
                self.tasks.retain(|t| t.id != id);
                self.recalculate_task_ids();
            }
            None => println!("[ERROR] Trying to delete unexisting task {id} \n"),
        }
    }

    pub fn order_completed(&mut self) {
        self.tasks.sort_by_key(|t| !t.completed);
        self.recalculate_task_ids();
    }
}
