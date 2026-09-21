use std::{
    collections::BTreeSet, error::Error, fmt::Display, fs::OpenOptions, hash::{DefaultHasher, Hash, Hasher}, io::{ErrorKind, Write},
};

use colored::Colorize;
use colorsys::{Hsl, Rgb};
use serde::{Deserialize, Serialize};

const DEFAULT_TAG: &str = "default";
const TODO_FILENAME: &str = ".todo_list";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Tag(String);
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let hue = (hasher.finish() % 360) as f64;
        let rgb: Rgb = Hsl::new(hue, 65.0, 60.0, None).into();
        let (r, g, b) = (rgb.red().round() as u8, rgb.green().round() as u8, rgb.blue().round() as u8);

        write!(f, "{}", format!("#{}", self.0).truecolor(r, g, b))
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
}

impl TaskManager {
    // ---------------------------
    // Private functions
    // ---------------------------
    fn next_id(&self) -> u32 {
        self.tasks.len() as u32 + 1
    }

    fn recalculate_task_ids(&mut self) {
        self.tasks.iter_mut()
            .enumerate()
            .for_each(|(i, task)| task.id = i as u32 + 1);
    }

    // TODO: Arreglar esto es lentisimo recorremos la lista
    // buscando tags cada ved que buscamos un elemento
    fn resolve_id(&self, id: u32) -> Option<u32> {
        match &self.selected_tag {
            // En default tag el id es el mismo
            None => Some(id),
            // En otros tags tenemos que encontrar el id real
            Some(tag) => {
                self.tasks.iter()
                    .filter(|t| t.tag.as_ref() == Some(tag))
                    .enumerate()
                    .find(|(i, _)| id == *i as u32 + 1)
                    .map(|(_, task)| task.id)
            }
        }
    }

    fn resolve_task(&self, id: u32) -> Option<&Task> {
        let real_id = self.resolve_id(id)?;
        self.tasks.iter().find(|t| t.id == real_id)
    }

    fn resolve_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        let real_id = self.resolve_id(id)?;
        self.tasks.iter_mut().find(|t| t.id == real_id)
    }
    // ---------------------------
    // Public functions
    // ---------------------------
    pub fn build() -> Result<Self, Box<dyn Error>> {
        let home = dirs::home_dir().ok_or("No se pudo determinar el directorio home")?;
        let path = home.join(TODO_FILENAME);
        let mut tasks = Vec::new();

        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Ok(Self { tasks, selected_tag: None });
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
        Ok(Self { tasks, selected_tag })
    }

    pub fn save_all(&self) -> Result<(), Box<dyn Error>> {
        let home = dirs::home_dir().ok_or("No se pudo determinar el directorio home")?;
        let path = home.join(TODO_FILENAME);
        let save_err = |e: &dyn Display| format!("No se pudo guardar el fichero {}: {e}", path.display());

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| save_err(&e))?;

        writeln!(file, "{}", self.selected_tag.as_ref().map(|t| t.0.as_str()).unwrap_or(DEFAULT_TAG))
            .map_err(|e| save_err(&e))?;

        let mut csv_wtr = csv::WriterBuilder::new().from_writer(file);
        for task in &self.tasks {
            csv_wtr.serialize(task).map_err(|e| save_err(&e))?;
        }
        csv_wtr.flush().map_err(|e| save_err(&e))?;
        Ok(())
    }

    // ---------------------------
    // Tasks management
    // ---------------------------
    pub fn add(&mut self, description: String, tag: Option<String>) {
        let tag = tag.map(Tag).or_else(|| self.selected_tag.clone());
        let task = Task::new(self.next_id(), description, tag);
        self.tasks.push(task);
    }

    pub fn list(&self, tags: Vec<String>) {
        let id_width = self.tasks.len().max(1).to_string().len();
        let wanted_tags: BTreeSet<Tag> = tags.into_iter().map(Tag).collect();

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

    pub fn mark_done(&mut self, ids: Vec<u32>) {
        for id in ids {
            match self.resolve_task_mut(id) {
                Some(task) => task.completed = true,
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
    }

    pub fn mark_undone(&mut self, ids: Vec<u32>) {
        for id in ids {
            match self.resolve_task_mut(id) {
                Some(task) => task.completed = false,
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
    }

    pub fn remove(&mut self, ids: Vec<u32>) {
        for id in ids{
            match self.resolve_task(id) {
                Some(delete_task) => {
                    println!("{}", format!("Deleted task {id}: {}", delete_task.description).cyan().bold());
                    self.tasks.retain(|t| t.id != id);
                }
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
        println!();
        self.recalculate_task_ids();
    }

    pub fn order_completed(&mut self) {
        self.tasks.sort_by_key(|t| !t.completed);
        self.tasks.sort_by(|a, b| a.tag.cmp(&b.tag));
        self.recalculate_task_ids();
    }
    // ---------------------------
    // Tag management 
    // ---------------------------
    pub fn tag(&mut self, id: u32, tag: String) {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => task.tag = Some(Tag(tag)),
            None => println!("{}", format!("Task {id} not found").bright_red().bold()),
        }
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

    pub fn remove_tag(&mut self, tags: Vec<String>) {
        for tag in tags {
            let tag = Tag(tag);
            println!("{}", format!("Deleting all occurrences of tasks with tag: {tag} \n").bold());
            self.tasks.retain(|t| t.tag.as_ref() != Some(&tag));
        }
        self.recalculate_task_ids();
    }

    pub fn rename_tag(&mut self, original: String, new: String) {
        let original = Tag(original);
        let new = Tag(new);
        println!("{}", format!("Renaming tag {original} to {new}").bold());
        self.tasks.iter_mut()
            .filter(|t| t.tag.as_ref() == Some(&original))
            .for_each(|t| t.tag = Some(new.clone()));
    }
}
