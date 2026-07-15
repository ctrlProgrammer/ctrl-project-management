use crate::dialogs::{show_delete_column_dialog, show_new_task_dialog};
use crate::models::{AppState, ColumnWidgets};
use crate::tasks::refresh_tasks;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::rc::Rc;

pub fn rebuild_columns(state: &Rc<AppState>) {
    while let Some(child) = state.kanban_box.first_child() {
        state.kanban_box.remove(&child);
    }
    *state.column_widgets.borrow_mut() = Vec::new();

    if let Some(project_id) = *state.current_project_id.borrow() {
        let db = state.db.borrow();
        if let Ok(columns) = db.get_columns_for_project(project_id) {
            drop(db);
            for (idx, col) in columns.iter().enumerate() {
                let cw = create_column_widgets(state, col.id, &col.name, idx);
                state.kanban_box.append(&cw.column_box);
                state.column_widgets.borrow_mut().push(cw);
            }
        }
    }

    state.kanban_box.append(&state.add_column_btn);
}

pub fn create_column_widgets(
    state: &Rc<AppState>,
    column_id: i64,
    column_name: &str,
    accent_index: usize,
) -> ColumnWidgets {
    let column_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    column_box.set_hexpand(true);
    column_box.set_vexpand(true);
    column_box.add_css_class("kanban-column");
    column_box.add_css_class(&format!("column-accent-{}", accent_index % 4));

    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    header_box.add_css_class("column-header-box");

    let header = gtk::Label::new(Some(column_name));
    header.add_css_class("column-header");
    header.set_xalign(0.0);
    header.set_hexpand(true);

    let delete_btn = gtk::Button::with_label("\u{00d7}");
    delete_btn.add_css_class("column-delete");
    delete_btn.set_has_frame(false);

    let s = state.clone();
    let col_id = column_id;
    let col_name = column_name.to_string();
    delete_btn.connect_clicked(move |_| {
        show_delete_column_dialog(&s, col_id, &col_name);
    });

    header_box.append(&header);
    header_box.append(&delete_btn);

    let count = gtk::Label::new(Some("0"));
    count.add_css_class("column-count");
    count.set_xalign(0.0);

    let list = gtk::ListBox::new();
    list.set_hexpand(true);
    list.set_vexpand(true);

    let drop_target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);
    let s = state.clone();
    let col_id = column_id;
    drop_target.connect_drop(move |_target, value, _x, _y| {
        let s = s.clone();
        if let Ok(task_id_str) = value.get::<String>() {
            if let Ok(task_id) = task_id_str.parse::<i64>() {
                let db = s.db.borrow();
                let _ = db.update_task_column(task_id, col_id);
                drop(db);
                refresh_tasks(&s);
            }
        }
        true
    });
    list.add_controller(drop_target);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_child(Some(&list));

    let add_btn = gtk::Button::with_label("+ Add Task");
    add_btn.add_css_class("add-task-button");
    add_btn.set_has_frame(false);

    let s = state.clone();
    let col_id = column_id;
    add_btn.connect_clicked(move |_| {
        show_new_task_dialog(&s, col_id);
    });

    column_box.append(&header_box);
    column_box.append(&count);
    column_box.append(&scroll);
    column_box.append(&add_btn);

    ColumnWidgets {
        id: column_id,
        list,
        count,
        column_box,
    }
}
