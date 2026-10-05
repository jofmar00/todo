use std::{
    collections::{BTreeMap, BTreeSet}, env, error::Error, fmt::Display, fs::OpenOptions, hash::{DefaultHasher, Hash, Hasher}, io::ErrorKind, os::unix::process::parent_id, path::PathBuf,
};

use colored::Colorize;
use colorsys::{Hsl, Rgb};
use csv::Reader;
use serde::{Deserialize, Serialize};

const DEFAULT_TAG: &str = "default";
const TODO_FILENAME: &str = ".todo";
const TODO_ENV: &str = "TODO_PATH";
const SESSION_ENV: &str = "TODO_SESSION_FILE";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Tag(String);
impl Tag {
    pub fn get_color(&self) -> (u8, u8, u8) {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let hue = (hasher.finish() % 360) as f64;
        let rgb: Rgb = Hsl::new(hue, 65.0, 60.0, None).into();
        (rgb.red().round() as u8, rgb.green().round() as u8, rgb.blue().round() as u8)
    }
}
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, g, b) = self.get_color();
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
        let status = if self.completed { "[x]".bright_green().bold() } else { "[ ]".bold() };
        write!(f, "{status}")?;

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

    fn resolve_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        let real_id = self.resolve_id(id)?;
        self.tasks.iter_mut().find(|t| t.id == real_id)
    }

    // El tag seleccionado es por sesión de terminal: se guarda en un fichero
    // temporal identificado por el PID de la shell (proceso padre), que es
    // el mismo para todos los `todo` lanzados desde esa terminal.
    // TODO_SESSION_FILE permite fijar otra ruta (lo usan los tests para no
    // compartir el tag entre procesos lanzados en paralelo).
    fn resolve_session_tag_path() -> PathBuf {
        match env::var_os(SESSION_ENV) {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => env::temp_dir().join(format!(".todo_tag_{}", parent_id())),
        }
    }

    // El fichero de tareas se busca en el directorio indicado por TODO_PATH,
    // y si no está definida (o está vacía) en el home del usuario.
    fn resolve_todo_path() -> Result<PathBuf, Box<dyn Error>> {
        let dir = match env::var_os(TODO_ENV) {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => dirs::home_dir().ok_or(format!("Could not get a proper path to create a .todo file, set the env {TODO_ENV} to a proper value to continue"))?,
        };
        Ok(dir.join(TODO_FILENAME))
    }

    fn parse_tasks(mut reader: Reader<&[u8]>) -> Result<Vec<Task>, Box<dyn Error>> {
        let mut tasks = Vec::new();
        for (i, result) in reader.deserialize().enumerate() {
            let mut task: Task = result.map_err(|e| format!("Could not parse .todo file: {e}"))?;
            task.id = i as u32 + 1;
            tasks.push(task);
        }
        Ok(tasks)
    }
    // ---------------------------
    // Public functions
    // ---------------------------
    pub fn build() -> Result<Self, Box<dyn Error>> {
        let path = Self::resolve_todo_path()?;
        let selected_tag = std::fs::read_to_string(Self::resolve_session_tag_path()).ok().map(Tag);

        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Self { tasks: Vec::new(), selected_tag }),
            Err(e) => return Err(format!("Could not read file {}: {e}", path.display()).into()),
        };

        // Load tasks in memory
        let csv_reader = csv::ReaderBuilder::new().from_reader(contents.as_bytes());
        let tasks = Self::parse_tasks(csv_reader)?;
        Ok(Self { tasks, selected_tag })
    }

    pub fn save_all(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::resolve_todo_path()?;
        let save_err = |e: &dyn Display| format!("Could not save file {}: {e}", path.display());

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| save_err(&e))?;

        match &self.selected_tag {
            Some(tag) => std::fs::write(Self::resolve_session_tag_path(), &tag.0).map_err(|e| save_err(&e))?,
            None => { let _ = std::fs::remove_file(Self::resolve_session_tag_path()); }
        }

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

        if self.selected_tag.is_none() && self.tasks.is_empty() {
            println!("No tasks inserted yet, try using {}", "todo help".italic().bold());
        }

        if let Some(selected_tag) = &self.selected_tag {
            println!("Using tag {selected_tag}");
        }

        // El id mostrado es el que aceptan los comandos: la posición dentro del
        // tag seleccionado. Las tareas de otros tags no son direccionables
        // desde aquí, así que se muestran con "-"
        let command_ids: BTreeMap<u32, u32> = self.tasks.iter()
            .filter(|task| self.selected_tag.is_none() || task.tag == self.selected_tag)
            .enumerate()
            .map(|(i, task)| (task.id, i as u32 + 1))
            .collect();

        self.tasks.iter()
            .filter(|task| if wanted_tags.is_empty() {
                self.selected_tag.is_none() || task.tag == self.selected_tag
            } else {
                task.tag.as_ref().is_some_and(|tag| wanted_tags.contains(tag))
            })
            .for_each(|task| match command_ids.get(&task.id) {
                Some(id) => println!("{id:>id_width$} {task}"),
                None => println!("{:>id_width$} {task}", "-"),
            });
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
        // Resolvemos todos los ids antes de borrar: al borrar una tarea las
        // posiciones dentro del tag se desplazan y apuntarían a otra tarea
        let resolved: Vec<(u32, Option<u32>)> = ids.into_iter()
            .map(|id| (id, self.resolve_id(id)))
            .collect();

        for (id, real_id) in resolved {
            match real_id.and_then(|real_id| self.tasks.iter().position(|t| t.id == real_id)) {
                Some(pos) => {
                    let delete_task = self.tasks.remove(pos);
                    println!("Deleted task {id}: {}", delete_task.description.bright_blue().bold());
                }
                None => println!("{}", format!("Task {id} not found").bright_red().bold()),
            }
        }
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
        match self.resolve_task_mut(id) {
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

    pub fn show_progress(&self) {
        const BAR_WIDTH: usize = 20;

        if self.tasks.is_empty() {
            println!("No tasks inserted yet, try using {}", "todo help".italic().bold());
            return;
        }

        let mut completed_per_tag: BTreeMap<Option<Tag>, (usize, usize)> = BTreeMap::new();
        for task in &self.tasks {
            let entry = completed_per_tag.entry(task.tag.clone()).or_insert((0, 0));
            entry.0 += task.completed as usize;
            entry.1 += 1;
        }

        let label_width = completed_per_tag.keys()
            .map(|tag| tag.as_ref().map_or(DEFAULT_TAG.len() + 1, |t| t.0.len() + 1))
            .max()
            .unwrap_or(0);

        for (tag, (done, total)) in completed_per_tag {
            // Tag color
            let (r, g, b) = match &tag {
                Some(tag) => tag.get_color(),
                None => (255, 255, 255),
            };

            let percent = (done * 100 / total) as u32;
            let filled = (BAR_WIDTH as f32 * percent as f32 / 100.0).round() as usize;
            let bar = format!("[{}{}]", "█".repeat(filled).truecolor(r, g, b).bold(), " ".repeat(BAR_WIDTH - filled));

            // El texto plano se rellena antes de colorear, ya que los códigos
            // ANSI cuentan como caracteres para el padding de `{:<width$}`.
            let (label, plain_len) = match tag {
                Some(tag) => (tag.to_string(), tag.0.len() + 1),
                None => (format!("#{DEFAULT_TAG}"), DEFAULT_TAG.len() + 1),
            };
            let padding = " ".repeat(label_width.saturating_sub(plain_len));
            println!("{label}{padding} {bar} {percent:>3}% ({done}/{total})");
        }
    }
}

#[cfg(test)]
mod tests;
