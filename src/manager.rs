use crate::{
    consts::{
        PROGRESS_BAR_WIDTH, SESSION_ENV_VAR, TAG_DEFAULT_NAME, TODO_FILENAME, TODO_PATH_ENV_VAR,
    },
    tag::Tag,
    task::{Task, TaskID},
};
use colored::Colorize;
use csv::Reader;
use std::{
    collections::{BTreeMap, BTreeSet}, default, env, fs, io, os, path::{Path, PathBuf},
};
use thiserror::Error;

pub struct TaskManager {
    tasks: Vec<Task>,
    selected_tag: Option<Tag>,
}
pub type TaskManagerResult = Result<(), TaskManagerError>;

impl TaskManager {
    fn next_id(&self) -> u32 {
        self.tasks.len() as u32 + 1
    }

    fn recalculate_task_ids(&mut self) {
        self.tasks
            .iter_mut()
            .enumerate()
            .for_each(|(i, task)| task.id = i as u32 + 1);
    }

    fn resolve_id(&self, id: TaskID) -> Option<TaskID> {
        match self.selected_tag {
            // En default tag el id es el mismo
            None => Some(id),
            // En otros tags tenemos que encontrar el id real
            Some(ref tag) => self
                .tasks
                .iter()
                .filter(|t| t.tag.as_ref() == Some(tag))
                .enumerate()
                .find(|(i, _)| id == *i as u32 + 1)
                .map(|(_, task)| task.id),
        }
    }

    fn get_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        let real_id = self.resolve_id(id)?;
        self.tasks.iter_mut().find(|t| t.id == real_id)
    }

    // TODO_SESSION_FILE can override the per-shell session file, for example
    // in tests or when running multiple shells attached to the same terminal.
    fn resolve_session_tag_path() -> PathBuf {
        match env::var_os(SESSION_ENV_VAR) {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            #[cfg(unix)]
            _ => env::temp_dir().join(format!(".todo_tag_{}", os::unix::process::parent_id())),
            #[cfg(not(unix))]
            _ => env::temp_dir().join(format!(".todo_tag_{}", std::process::id())),
        }
    }

    fn todo_path() -> Result<PathBuf, TaskManagerError> {
        let dir = env::var_os(TODO_PATH_ENV_VAR)
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .or_else(dirs::home_dir)
            .ok_or(TaskManagerError::FileNotFound)?;

        Ok(dir.join(TODO_FILENAME))
    }

    fn todo_tag() -> Result<Option<Tag>, TaskManagerError> {
        let tag = Self::read_session_tag(&Self::resolve_session_tag_path())?;
        Ok(tag.filter(|tag| *tag != Tag::default()))
    }

    fn read_session_tag(path: &Path) -> Result<Option<Tag>, TaskManagerError> {
        match fs::read_to_string(path) {
            Ok(tag) if !tag.is_empty() => Ok(Some(Tag(tag))),
            Ok(_) => Ok(None),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(TaskManagerError::SessionTagError(error)),
        }
    }

    fn write_session_tag(path: &Path, tag: Option<&Tag>) -> TaskManagerResult {
        match tag {
            Some(tag) => fs::write(path, &tag.0).map_err(TaskManagerError::SessionTagError),
            None => match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(TaskManagerError::SessionTagError(error)),
            },
        }
    }

    fn load_tasks(mut reader: Reader<&[u8]>) -> Result<Vec<Task>, TaskManagerError> {
        let mut tasks = Vec::new();
        for (i, result) in reader.deserialize().enumerate() {
            let mut task: Task = result.map_err(TaskManagerError::TaskParsingError)?;
            task.id = i as u32 + 1;
            tasks.push(task);
        }
        Ok(tasks)
    }
    // ---------------------------
    // Public functions
    // ---------------------------
    pub fn build() -> Result<Self, TaskManagerError> {
        let todo_file_contents = match fs::read_to_string(Self::todo_path()?) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            other => other.map_err(TaskManagerError::TaskLoadError)?,
        };

        let csv_reader = csv::ReaderBuilder::new().from_reader(todo_file_contents.as_bytes());
        let tasks = Self::load_tasks(csv_reader)?;
        let selected_tag = Self::todo_tag()?;
        Ok(Self {
            tasks,
            selected_tag,
        })
    }

    pub fn save_tasks(&self) -> TaskManagerResult {
        let path = Self::todo_path()?;

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(TaskManagerError::SaveError)?;

        let mut writer = csv::WriterBuilder::new().from_writer(file);
        self.tasks
            .iter()
            .try_for_each(|task| writer.serialize(task))
            .map_err(TaskManagerError::TaskParsingError)?;
        writer.flush().map_err(TaskManagerError::SaveError)
    }

    // ---------------------------
    // Tasks management
    // ---------------------------
    pub fn add(&mut self, description: String, tag: Option<Tag>) {
        let tag = tag.or_else(|| self.selected_tag.clone());
        let task = Task::new(self.next_id(), description, tag);
        self.tasks.push(task);
    }

    pub fn list(&self, tags: Vec<Tag>) -> TaskManagerResult {
        let id_width = self.tasks.len().max(1).to_string().len();
        let wanted_tags: BTreeSet<Tag> = tags.into_iter().collect();

        if self.tasks.is_empty() {
            return Err(TaskManagerError::NoTasksError);
        }

        if let Some(ref selected_tag) = self.selected_tag {
            println!("Using tag {selected_tag}");
        }

        // El id mostrado es el que aceptan los comandos: la posición dentro del
        // tag seleccionado. Las tareas de otros tags no son direccionables
        // desde aquí, así que se muestran con "-"
        let command_ids: BTreeMap<u32, u32> = self
            .tasks
            .iter()
            .filter(|task| self.selected_tag.is_none() || task.tag == self.selected_tag)
            .enumerate()
            .map(|(i, task)| (task.id, i as u32 + 1))
            .collect();

        self.tasks
            .iter()
            .filter(|task| {
                if wanted_tags.is_empty() {
                    self.selected_tag.is_none() || task.tag == self.selected_tag
                } else {
                    task.tag
                        .as_ref()
                        .is_some_and(|tag| wanted_tags.contains(tag))
                }
            })
            .for_each(|task| match command_ids.get(&task.id) {
                Some(id) => println!("{id:>id_width$} {task}"),
                None => println!("{:>id_width$} {task}", "-"),
            });

        Ok(())
    }

    pub fn mark_done(&mut self, ids: Vec<TaskID>) -> TaskManagerResult {
        for id in ids {
            let task = self
                .get_task_mut(id)
                .ok_or(TaskManagerError::TaskNotFound(id))?;
            task.completed = true;
        }
        Ok(())
    }

    pub fn mark_undone(&mut self, ids: Vec<TaskID>) -> TaskManagerResult {
        for id in ids {
            let task = self
                .get_task_mut(id)
                .ok_or(TaskManagerError::TaskNotFound(id))?;
            task.completed = false;
        }
        Ok(())
    }

    pub fn remove(&mut self, ids: Vec<TaskID>) -> TaskManagerResult {
        // Resolvemos todos los ids antes de borrar: al borrar una tarea las
        // posiciones dentro del tag se desplazan y apuntarían a otra tarea
        let resolved: Vec<(TaskID, Option<TaskID>)> = ids
            .into_iter()
            .map(|id| (id, self.resolve_id(id)))
            .collect();

        for (id, real_id) in resolved {
            match real_id.and_then(|real_id| self.tasks.iter().position(|t| t.id == real_id)) {
                Some(pos) => {
                    let delete_task = self.tasks.remove(pos);
                    println!(
                        "Deleted task {id}: {}",
                        delete_task.description.bright_blue().bold()
                    );
                }
                None => return Err(TaskManagerError::TaskNotFound(id)),
            }
        }
        self.recalculate_task_ids();
        Ok(())
    }

    pub fn order_completed(&mut self) {
        self.tasks.sort_by_key(|t| !t.completed);
        self.tasks.sort_by(|a, b| a.tag.cmp(&b.tag));
        self.recalculate_task_ids();
    }
    // ---------------------------
    // Tag management
    // ---------------------------
    pub fn tag(&mut self, id: TaskID, tag: Tag) -> TaskManagerResult {
        match self.get_task_mut(id) {
            Some(task) => task.tag = Some(tag),
            None => return Err(TaskManagerError::TaskNotFound(id)),
        }
        Ok(())
    }

    pub fn list_tags(&self) {
        println!("{}", Tag::default());
        self.tasks
            .iter()
            .filter_map(|t| t.tag.as_ref())
            .collect::<BTreeSet<&Tag>>()
            .into_iter()
            .for_each(|tag| {
                let current = if Some(tag) == self.selected_tag.as_ref() {
                    " (Current)"
                } else {
                    ""
                };
                println!("{tag}{current}");
            });
    }

    pub fn use_tag(&mut self, tag: Option<Tag>) -> TaskManagerResult {
        let selected_tag = tag.filter(|t| *t != Tag::default());
        Self::write_session_tag(&Self::resolve_session_tag_path(), selected_tag.as_ref())?;
        self.selected_tag = selected_tag;
        if self.selected_tag.is_none() {
            println!("Using tag {}", Tag::default())
        };
        Ok(())
    }

    pub fn remove_tag(&mut self, tags: Vec<Tag>) {
        for tag in tags {
            println!(
                "{}",
                format!("Deleting all occurrences of tasks with tag: {tag} \n").bold()
            );
            self.tasks.retain(|t| t.tag.as_ref() != Some(&tag));
        }
        self.recalculate_task_ids();
    }

    pub fn rename_tag(&mut self, original: Tag, new: Tag) {
        println!("{}", format!("Renaming tag {original} to {new}").bold());
        self.tasks
            .iter_mut()
            .filter(|t| t.tag.as_ref() == Some(&original))
            .for_each(|t| t.tag = Some(new.clone()));
    }

    pub fn show_progress(&self) -> TaskManagerResult {
        if self.tasks.is_empty() {
            return Err(TaskManagerError::NoTasksError);
        }

        let mut completed_per_tag: BTreeMap<Option<Tag>, (usize, usize)> = BTreeMap::new();
        for task in &self.tasks {
            let entry = completed_per_tag.entry(task.tag.clone()).or_insert((0, 0));
            entry.0 += task.completed as usize;
            entry.1 += 1;
        }

        let label_width = completed_per_tag
            .keys()
            .map(|tag| {
                tag.as_ref()
                    .map_or(TAG_DEFAULT_NAME.len() + 1, |t| t.0.len() + 1)
            })
            .max()
            .unwrap_or(0);

        for (tag, (done, total)) in completed_per_tag {
            // Tag color
            let color = match &tag {
                Some(tag) => tag.color(),
                None => colored::CustomColor {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            };

            let percent = (done * 100 / total) as u32;
            let filled = (PROGRESS_BAR_WIDTH as f32 * percent as f32 / 100.0).round() as usize;
            let bar = format!(
                "[{}{}]",
                "█".repeat(filled).custom_color(color).bold(),
                " ".repeat(PROGRESS_BAR_WIDTH - filled)
            );

            // El texto plano se rellena antes de colorear, ya que los códigos
            // ANSI cuentan como caracteres para el padding de `{:<width$}`.
            let (label, plain_len) = match tag {
                Some(tag) => (tag.to_string(), tag.0.len() + 1),
                None => (format!("{}", Tag::default()), TAG_DEFAULT_NAME.len() + 1),
            };
            let padding = " ".repeat(label_width.saturating_sub(plain_len));
            println!("{label}{padding} {bar} {percent:>3}% ({done}/{total})");
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum TaskManagerError {
    #[error("no tasks inserted yet, try using {}", "todo help".italic().bold())]
    NoTasksError,

    #[error(
        "could not get a proper path to create a .todo file, set the env {TODO_PATH_ENV_VAR} to a proper path to continue"
    )]
    FileNotFound,

    #[error("task with id {0} not found")]
    TaskNotFound(TaskID),

    #[error("tasks couldn't be saved: {0}")]
    SaveError(io::Error),

    #[error("couldn't load tasks: {0}")]
    TaskLoadError(io::Error),

    #[error("couldn't read or save the session tag: {0}")]
    SessionTagError(io::Error),

    #[error("couldn't parse task: {0}")]
    TaskParsingError(csv::Error),
}
