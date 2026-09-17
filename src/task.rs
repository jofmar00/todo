use std::{
    collections::{BTreeSet, HashSet}, env, error::Error, fmt::Display, fs::OpenOptions, io::{ErrorKind, Write}, path::{Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

const DEFAULT_TAG: &str = "default";
const TODO_FILENAME: &str = ".todo_list";

// TODO: Cambiar esto
fn todo_path() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .map_err(|_| "No se pudo determinar el directorio home (variable HOME no definida)".to_string())?;
    Ok(PathBuf::from(home).join(TODO_FILENAME))
}

fn save_err(path: &Path, e: impl Display) -> String {
    format!("No se pudo guardar el fichero {}: {e}", path.display())
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Tag(String);
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, g, b) = self.get_color();
        write!(f, "{}", format!("#{}", self.0).truecolor(r, g, b))
    }
}

impl Tag {
    fn as_str(&self) -> &str {
        &self.0
    }

    fn get_color(&self) -> (u8, u8, u8) {
        let hash = self.0.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let hue = (hash % 360) as f32;
        hsl_to_rgb(hue, 0.65, 0.6)
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    #[serde(skip)]
    id: u32,
    description: String,
    completed: bool,
    tag: Option<Tag>
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id_width = f.width().unwrap_or(1);
        let status = if self.completed { "[x]".bright_green().bold() } else { "[ ]".bold() };
        write!(f, "{:>id_width$} {status} ", self.id)?;

        if let Some(tag) = &self.tag {
            write!(f, "({})", tag)?;
        }
        else {
            write!(f, "()")?;
        }

        let description = if self.completed {
            self.description.strikethrough().dimmed()
        } else {
            self.description.normal()
        };
        write!(f, " {description}")?;
        Ok(())
    }
}

impl Task {
    fn new(id: u32, description: String, tag: Option<Tag>) -> Self {
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
    selected_tag: Option<Tag>,
    path: PathBuf,
}

impl TaskManager {
    // ---------------------------
    // Private functions
    // ---------------------------
    fn next_id(&self) -> u32 {
        self.tasks.len() as u32 + 1
    }

    fn recalculate_task_ids(&mut self) {
        for (i, task) in self.tasks.iter_mut().enumerate() {
            task.id = i as u32 + 1;
        }
    }

    // ---------------------------
    // Public functions
    // ---------------------------
    pub fn build() -> Result<Self, Box<dyn Error>> {
        let path = todo_path()?;
        let mut tasks = Vec::new();

        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Ok(Self { tasks, selected_tag: None, path });
            }
            Err(e) => return Err(format!("No se pudo leer el fichero {}: {e}", path.display()).into()),
        };

        // Obtain tag
        let mut lines = contents.lines();
        let selected_tag = lines.next().filter(|l| *l != DEFAULT_TAG).map(String::from).map(Tag);
        let rest = lines.collect::<Vec<_>>().join("\n");

        // Load tasks in memory
        let mut csv_reader = csv::ReaderBuilder::new().from_reader(rest.as_bytes());
        for (i, result) in csv_reader.deserialize().enumerate() {
            let mut task: Task = result
                .map_err(|e| format!("No se pudo parsear el CSV de {}: {e}", path.display()))?;
            task.id = i as u32 + 1;
            tasks.push(task);
        }
        Ok(Self { tasks, selected_tag, path })
    }

    pub fn save_all(&self) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| save_err(&self.path, e))?;

        writeln!(file, "{}", self.selected_tag.as_ref().map(Tag::as_str).unwrap_or(DEFAULT_TAG))
            .map_err(|e| save_err(&self.path, e))?;

        let mut csv_wtr = csv::WriterBuilder::new().from_writer(file);
        for task in &self.tasks {
            csv_wtr.serialize(task).map_err(|e| save_err(&self.path, e))?;
        }
        csv_wtr.flush().map_err(|e| save_err(&self.path, e))?;
        Ok(())
    }


    pub fn add(&mut self, description: String, tag: Option<String>) {
        let tag = tag.map(Tag).or_else(|| self.selected_tag.clone());
        let task = Task::new(self.next_id(), description, tag);
        self.tasks.push(task);
    }

    pub fn list(&self, tags: Vec<String>) {
        let id_width = self.tasks.len().max(1).to_string().len();
        let wanted_tags: HashSet<Tag> = tags.into_iter().map(Tag).collect();

        if let Some(selected_tag) = &self.selected_tag {
            println!("Using tag {selected_tag}");
        }
        
        self.tasks.iter()
            .filter(|task| if wanted_tags.is_empty() {
                self.selected_tag.is_none() || task.tag == self.selected_tag
            } else {
                task.tag.as_ref().is_some_and(|tag| wanted_tags.contains(tag))
            })
            .for_each(|task| println!("{task:id_width$}"));
    }

    pub fn list_tags(&self) {
        println!("#{DEFAULT_TAG}");
        self.tasks.iter()
            .filter_map(|t| t.tag.as_ref())
            // Pasamos a BTreeSet para evitar duplicados
            .collect::<BTreeSet<&Tag>>()
            .into_iter()
            .for_each(|tag| { 
                let current = if Some(tag) == self.selected_tag.as_ref() { " (Current)" } else { "" };
                println!("{tag}{current}");
            });
    }

    pub fn use_tag(&mut self, tag: Option<String>) {
        self.selected_tag = tag.filter(|t| t != DEFAULT_TAG).map(Tag);
        if self.selected_tag.is_none() { println!("Using tag {DEFAULT_TAG}")};
    }

    pub fn mark_done(&mut self, ids: Vec<u32>) {
        for id in ids {
            match self.tasks.iter_mut().find(|t| t.id == id) {
                Some(task) => task.completed = true,
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
    }

    pub fn mark_undone(&mut self, ids: Vec<u32>) {
        for id in ids {
            match self.tasks.iter_mut().find(|t| t.id == id) {
                Some(task) => task.completed = false,
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
    }

    pub fn tag(&mut self, id: u32, tag: String) {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => task.tag = Some(Tag(tag)),
            None => println!("{}", format!("Task {id} not found").bright_red().bold()),
        }
    }


    pub fn remove(&mut self, ids: Vec<u32>) {
        for id in ids{
            match self.tasks.iter().find(|t| t.id == id) {
                Some(delete_task) => {
                    println!("{}", format!("Deleted task {id}: {} \n", delete_task.description).cyan().bold());
                    self.tasks.retain(|t| t.id != id);
                    self.recalculate_task_ids();
                }
                None => println!("{}", format!("Task {id} not found\n").bright_red().bold()),
            }
        }
    }

    pub fn remove_tag(&mut self, tags: Vec<String>) {
        for tag in tags {
            let tag = Tag(tag);
            println!("{}", format!("Deleting all occurrences of tasks with tag: {tag} \n").bold());
            self.tasks.retain(|t| t.tag.as_ref() != Some(&tag));
            self.recalculate_task_ids();
        }
    }

    pub fn order_completed(&mut self) {
        self.tasks.sort_by_key(|t| !t.completed);
        self.tasks.sort_by(|a, b| a.tag.cmp(&b.tag));
        self.recalculate_task_ids();
    }

}
