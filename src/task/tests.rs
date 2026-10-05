use super::*;

// ---------------------------
// Helpers
// ---------------------------
fn manager(selected: Option<&str>, tasks: &[(&str, Option<&str>)]) -> TaskManager {
    let tasks = tasks.iter()
        .enumerate()
        .map(|(i, (description, tag))| Task::new(i as u32 + 1, description.to_string(), tag.map(|t| Tag(t.into()))))
        .collect();
    TaskManager { tasks, selected_tag: selected.map(|t| Tag(t.into())) }
}

fn descriptions(tm: &TaskManager) -> Vec<&str> {
    tm.tasks.iter().map(|t| t.description.as_str()).collect()
}

fn ids(tm: &TaskManager) -> Vec<u32> {
    tm.tasks.iter().map(|t| t.id).collect()
}

fn tag_of<'a>(tm: &'a TaskManager, description: &str) -> Option<&'a str> {
    tm.tasks.iter()
        .find(|t| t.description == description)
        .and_then(|t| t.tag.as_ref())
        .map(|t| t.0.as_str())
}

// A(work) B() C(work) D(work)
fn mixed(selected_tag: Option<&str>) -> TaskManager {
    manager(selected_tag,
            &[("A", Some("work")),
              ("B", None),
              ("C", Some("work")),
              ("D", Some("work"))])
}

// ---------------------------
// resolve_id
// ---------------------------
#[test]
fn resolve_id_without_tag_is_the_real_id() {
    let tm = mixed(None);
    assert_eq!(tm.resolve_id(2), Some(2));
}

#[test]
fn resolve_id_with_tag_uses_position_inside_the_tag() {
    let tm = mixed(Some("work"));
    assert_eq!(tm.resolve_id(1), Some(1)); // A
    assert_eq!(tm.resolve_id(2), Some(3)); // C
    assert_eq!(tm.resolve_id(3), Some(4)); // D
}

#[test]
fn resolve_id_out_of_range_is_none() {
    let tm = mixed(Some("work"));
    assert_eq!(tm.resolve_id(0), None);
    assert_eq!(tm.resolve_id(4), None);
}

// ---------------------------
// Tasks management
// ---------------------------
#[test]
fn add_uses_selected_tag_by_default() {
    let mut tm = mixed(Some("work"));
    tm.add("E".into(), None);
    assert_eq!(tag_of(&tm, "E"), Some("work"));
    assert_eq!(tm.tasks.last().unwrap().id, 5);
}

#[test]
fn add_with_explicit_tag_overrides_selected_tag() {
    let mut tm = mixed(Some("work"));
    tm.add("E".into(), Some("home".into()));
    assert_eq!(tag_of(&tm, "E"), Some("home"));
}

#[test]
fn done_and_undone_with_tag_resolve_local_ids() {
    let mut tm = mixed(Some("work"));
    tm.mark_done(vec![2]);
    assert!(tm.tasks[2].completed, "C should be completed");
    assert!(!tm.tasks[1].completed, "B should not be touched");

    tm.mark_undone(vec![2]);
    assert!(!tm.tasks[2].completed);
}

#[test]
fn remove_several_ids_with_tag_removes_the_listed_tasks() {
    // Regresión: al borrar la primera las posiciones se desplazaban
    let mut tm = mixed(Some("work"));
    tm.remove(vec![1, 2]);
    assert_eq!(descriptions(&tm), ["B", "D"]);
}

#[test]
fn remove_repeated_id_removes_only_once() {
    let mut tm = mixed(None);
    tm.remove(vec![1, 1]);
    assert_eq!(descriptions(&tm), ["B", "C", "D"]);
}

#[test]
fn remove_unknown_id_keeps_tasks() {
    let mut tm = mixed(None);
    tm.remove(vec![9]);
    assert_eq!(descriptions(&tm), ["A", "B", "C", "D"]);
}

#[test]
fn remove_recalculates_ids() {
    let mut tm = mixed(None);
    tm.remove(vec![2]);
    assert_eq!(ids(&tm), [1, 2, 3]);
}

#[test]
fn order_completed_puts_completed_first_inside_each_tag() {
    let mut tm = mixed(None);
    tm.mark_done(vec![4]); // D
    tm.order_completed();
    assert_eq!(descriptions(&tm), ["B", "D", "A", "C"]);
    assert_eq!(ids(&tm), [1, 2, 3, 4]);
}

// ---------------------------
// Tag management
// ---------------------------
#[test]
fn tag_with_selected_tag_resolves_local_id() {
    // Regresión: antes usaba el id real y cambiaba otra tarea
    let mut tm = mixed(Some("work"));
    tm.tag(2, "home".into());
    assert_eq!(tag_of(&tm, "C"), Some("home"));
    assert_eq!(tag_of(&tm, "B"), None);
}

#[test]
fn use_default_tag_clears_selection() {
    let mut tm = mixed(Some("work"));
    tm.use_tag(Some(DEFAULT_TAG.into()));
    assert!(tm.selected_tag.is_none());

    tm.use_tag(Some("home".into()));
    assert!(tm.selected_tag == Some(Tag("home".into())));
}

#[test]
fn remove_tag_removes_its_tasks_and_recalculates_ids() {
    let mut tm = mixed(None);
    tm.remove_tag(vec!["work".into()]);
    assert_eq!(descriptions(&tm), ["B"]);
    assert_eq!(ids(&tm), [1]);
}

#[test]
fn rename_tag_only_changes_matching_tasks() {
    let mut tm = manager(None, &[("A", Some("work")), ("B", Some("home"))]);
    tm.rename_tag("work".into(), "job".into());
    assert_eq!(tag_of(&tm, "A"), Some("job"));
    assert_eq!(tag_of(&tm, "B"), Some("home"));
}

// ---------------------------
// Parsing
// ---------------------------
#[test]
fn parse_tasks_assigns_sequential_ids() {
    let csv = "description,completed,tag\nA,false,work\nB,true,\n";
    let tasks = TaskManager::parse_tasks(csv::Reader::from_reader(csv.as_bytes())).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, 1);
    assert_eq!(tasks[1].id, 2);
    assert!(tasks[1].completed);
    assert!(tasks[1].tag.is_none());
}

#[test]
fn parse_tasks_fails_on_invalid_csv() {
    let csv = "description,completed,tag\nA,maybe,work\n";
    let result = TaskManager::parse_tasks(csv::Reader::from_reader(csv.as_bytes()));
    assert!(result.is_err());
}
