use crate::columns::rebuild_columns;
use crate::models::{AppState, ProjectInfo};
use crate::tasks::refresh_tasks;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::rc::Rc;

pub fn refresh_projects(state: &Rc<AppState>) {
    let db = state.db.borrow();
    let project_list = db.get_all_projects().unwrap_or_default();
    drop(db);

    let info: Vec<ProjectInfo> = project_list
        .iter()
        .map(|(id, name, description)| ProjectInfo { id: *id, name: name.clone(), description: description.clone() })
        .collect();
    *state.projects.borrow_mut() = info;

    while let Some(child) = state.project_listbox.first_child() {
        state.project_listbox.remove(&child);
    }

    let projects = state.projects.borrow();
    if projects.is_empty() {
        drop(projects);
        *state.current_project_id.borrow_mut() = None;
        state.kanban_box.remove(&state.add_column_btn);
        while let Some(child) = state.kanban_box.first_child() {
            state.kanban_box.remove(&child);
        }
        *state.column_widgets.borrow_mut() = Vec::new();
        state.project_selector.set_label("No projects");
        state.project_search.set_text("");
        return;
    }

    for project in projects.iter() {
        let row = gtk::ListBoxRow::new();
        let label = gtk::Label::new(Some(&project.name));
        label.set_xalign(0.0);
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        label.set_margin_bottom(8);
        row.set_child(Some(&label));

        state.project_listbox.append(&row);
    }
    drop(projects);

    state.project_selector.set_label(&project_list[0].1);
    *state.current_project_id.borrow_mut() = Some(project_list[0].0);
    state.project_search.set_text("");
    rebuild_columns(state);
    refresh_tasks(state);
}
