use crate::columns::rebuild_columns;
use crate::models::AppState;
use crate::projects::refresh_projects;
use crate::tasks::refresh_tasks;
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::cell::RefCell;
use std::rc::Rc;

fn make_buttons(cancel_label: &str, action_label: &str) -> (gtk::Button, gtk::Button, gtk::Box) {
    let cancel = gtk::Button::with_label(cancel_label);
    let action = gtk::Button::with_label(action_label);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.set_halign(gtk::Align::End);

    row.append(&cancel);
    row.append(&action);
    (cancel, action, row)
}

pub fn show_new_project_dialog(state: &Rc<AppState>) {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("New Project"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(320, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    dialog.add_css_class("dialog-window");

    let content = dialog.content_area();
    content.add_css_class("dialog-content");

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.set_margin_top(6);
    card.set_margin_bottom(20);
    card.set_margin_start(20);
    card.set_margin_end(20);

    let title_label = gtk::Label::new(Some("New Project"));
    title_label.add_css_class("dialog-title");
    title_label.set_halign(gtk::Align::Start);
    card.append(&title_label);

    let title_entry = gtk::Entry::new();
    title_entry.set_placeholder_text(Some("Project title"));
    title_entry.add_css_class("dialog-entry");
    card.append(&title_entry);

    let desc_scroll = gtk::ScrolledWindow::new();
    desc_scroll.set_min_content_height(80);
    desc_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    desc_scroll.add_css_class("dialog-entry");
    let desc_buffer = gtk::TextBuffer::new(None);
    let desc_view = gtk::TextView::with_buffer(&desc_buffer);
    desc_view.set_wrap_mode(gtk::WrapMode::Word);
    desc_view.set_top_margin(6);
    desc_view.set_left_margin(10);
    desc_view.set_right_margin(10);
    desc_view.add_css_class("dialog-textview");
    desc_scroll.set_child(Some(&desc_view));
    card.append(&desc_scroll);

    let (cancel_btn, create_btn, buttons) = make_buttons("Cancel", "New Project");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_create = dialog.clone();
    let title_entry_for_resp = title_entry.clone();
    let desc_buffer_for_resp = desc_buffer.clone();
    create_btn.connect_clicked(move |_| {
        let title = title_entry_for_resp.text().to_string().trim().to_string();
        if !title.is_empty() {
            let start = desc_buffer_for_resp.start_iter();
            let end = desc_buffer_for_resp.end_iter();
            let description = desc_buffer_for_resp.text(&start, &end, false).to_string().trim().to_string();
            let db = s.db.borrow();
            if let Ok(id) = db.create_project(&title, &description) {
                *s.current_project_id.borrow_mut() = Some(id);
                drop(db);
                refresh_projects(&s);
            }
            dialog_for_create.close();
        }
    });

    card.append(&buttons);
    content.append(&card);

    let dialog_activate = create_btn.clone();
    title_entry.connect_activate(move |_| {
        dialog_activate.emit_clicked();
    });

    dialog.present();
    title_entry.grab_focus();
}

pub fn show_delete_project_dialog(state: &Rc<AppState>) {
    if state.current_project_id.borrow().is_none() {
        return;
    }

    let project_name = state
        .project_selector
        .label()
        .unwrap_or_default()
        .to_string();

    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Delete Project"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(320, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    dialog.add_css_class("dialog-window");

    let content = dialog.content_area();
    content.add_css_class("dialog-content");

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.set_margin_top(6);
    card.set_margin_bottom(20);
    card.set_margin_start(20);
    card.set_margin_end(20);

    let title_label = gtk::Label::new(Some("Delete Project"));
    title_label.add_css_class("dialog-title");
    title_label.set_halign(gtk::Align::Start);
    card.append(&title_label);

    let label = gtk::Label::new(Some(&format!(
        "Delete \"{}\" and all its columns and tasks?\nThis cannot be undone.",
        project_name
    )));
    label.set_halign(gtk::Align::Start);
    label.set_wrap(true);
    card.append(&label);

    let (cancel_btn, delete_btn, buttons) = make_buttons("Cancel", "Delete");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_delete = dialog.clone();
    delete_btn.connect_clicked(move |_| {
        let pid = *s.current_project_id.borrow();
        if let Some(pid) = pid {
            let db = s.db.borrow();
            let _ = db.delete_project(pid);
            drop(db);
            *s.current_project_id.borrow_mut() = None;
            refresh_projects(&s);
        }
        dialog_for_delete.close();
    });

    card.append(&buttons);
    content.append(&card);

    dialog.present();
}

pub fn show_new_task_dialog(state: &Rc<AppState>, column_id: i64) {
    if state.current_project_id.borrow().is_none() {
        return;
    }

    let col_name = {
        let cw = state.column_widgets.borrow();
        cw.iter()
            .find(|c| c.id == column_id)
            .map(|c| {
                let label: Option<gtk::Label> = c
                    .column_box
                    .first_child()
                    .and_then(|h| h.first_child())
                    .and_then(|l| l.downcast::<gtk::Label>().ok());
                label
                    .as_ref()
                    .map(|l| l.text().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    };

    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("New Task"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(400, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    dialog.add_css_class("dialog-window");

    let content = dialog.content_area();
    content.add_css_class("dialog-content");

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.set_margin_top(6);
    card.set_margin_bottom(20);
    card.set_margin_start(20);
    card.set_margin_end(20);

    let title_label = gtk::Label::new(Some("New Task"));
    title_label.add_css_class("dialog-title");
    title_label.set_halign(gtk::Align::Start);
    card.append(&title_label);

    let label = gtk::Label::new(Some(&format!("Add to: {}", col_name)));
    label.set_margin_bottom(4);
    label.set_halign(gtk::Align::Start);
    card.append(&label);

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Task title"));
    entry.add_css_class("dialog-entry");
    card.append(&entry);

    let desc_scroll = gtk::ScrolledWindow::new();
    desc_scroll.set_min_content_height(80);
    desc_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    desc_scroll.add_css_class("dialog-entry");
    let desc_buffer = gtk::TextBuffer::new(None);
    let desc_view = gtk::TextView::with_buffer(&desc_buffer);
    desc_view.set_wrap_mode(gtk::WrapMode::Word);
    desc_view.set_top_margin(6);
    desc_view.set_left_margin(10);
    desc_view.set_right_margin(10);
    desc_view.add_css_class("dialog-textview");
    desc_view.set_hexpand(true);
    desc_view.set_vexpand(true);
    desc_scroll.set_child(Some(&desc_view));
    card.append(&desc_scroll);

    let link_entry = gtk::Entry::new();
    link_entry.set_placeholder_text(Some("Link URL (optional)"));
    link_entry.add_css_class("dialog-entry");
    card.append(&link_entry);

    let tag_entry = gtk::Entry::new();
    tag_entry.set_placeholder_text(Some("Tags (comma separated, type to see existing)"));
    tag_entry.add_css_class("dialog-entry");
    card.append(&tag_entry);

    let due_date_entry = gtk::Entry::new();
    due_date_entry.set_placeholder_text(Some("Due date (YYYY-MM-DD)"));
    due_date_entry.add_css_class("dialog-entry");
    card.append(&due_date_entry);

    let priority_store = gtk::StringList::new(&["Normal", "Low", "Medium", "High", "Critical"]);
    let priority_dropdown = gtk::DropDown::new(
        Some(priority_store.upcast::<gio::ListModel>()),
        Option::<&gtk::Expression>::None,
    );
    priority_dropdown.add_css_class("dialog-entry");
    card.append(&priority_dropdown);

    let completion = gtk::EntryCompletion::new();
    let tag_model = gtk::ListStore::new(&[glib::Type::STRING]);
    completion.set_model(Some(&tag_model));
    completion.set_text_column(0);
    tag_entry.set_completion(Some(&completion));

    if let Some(project_id) = *state.current_project_id.borrow() {
        if let Ok(existing) = state.db.borrow().get_existing_tags(project_id) {
            for tag in &existing {
                tag_model.insert_with_values(None, &[(0, &tag as &dyn ToValue)]);
            }
        }
    }

    let docs_label = gtk::Label::new(Some("No documents attached"));
    docs_label.add_css_class("dialog-docs-label");
    docs_label.set_halign(gtk::Align::Start);
    card.append(&docs_label);

    let docs = Rc::new(RefCell::new(Vec::new()));

    let drop_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
    drop_area.set_size_request(-1, 60);
    drop_area.add_css_class("dialog-drop-area");
    let drop_hint = gtk::Label::new(Some("Drop files here"));
    drop_area.append(&drop_hint);
    card.append(&drop_area);

    let docs_for_drop = docs.clone();
    let docs_label_for_drop = docs_label.clone();
    let formats = gdk::ContentFormats::new(&["text/uri-list"]);
    let drop_target = gtk::DropTarget::builder()
        .formats(&formats)
        .actions(gdk::DragAction::COPY)
        .build();
    drop_target.connect_drop(move |_target, value, _x, _y| {
        if let Ok(uris) = value.get::<String>() {
            let paths: Vec<String> = uris
                .split(|c: char| c == '\n' || c == '\r')
                .filter(|s| !s.is_empty())
                .filter(|u| u.starts_with("file://"))
                .map(|u| u.trim_start_matches("file://").to_string())
                .collect();
            if !paths.is_empty() {
                let mut docs = docs_for_drop.borrow_mut();
                docs.extend(paths);
                let count = docs.len();
                docs_label_for_drop.set_text(&format!("{} document{} attached", count, if count == 1 { "" } else { "s" }));
            }
        }
        true
    });
    drop_area.add_controller(drop_target);

    let (cancel_btn, add_btn, buttons) = make_buttons("Cancel", "Add");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_add = dialog.clone();
    let entry_for_resp = entry.clone();
    let link_entry_for_resp = link_entry.clone();
    let tag_entry_for_resp = tag_entry.clone();
    let docs_for_add = docs.clone();
    let desc_buffer_for_add = desc_buffer.clone();
    let due_date_entry_for_resp = due_date_entry.clone();
    let priority_dropdown_for_resp = priority_dropdown.clone();
    add_btn.connect_clicked(move |_| {
        let title = entry_for_resp.text().to_string().trim().to_string();
        if !title.is_empty() {
            let start = desc_buffer_for_add.start_iter();
            let end = desc_buffer_for_add.end_iter();
            let description = desc_buffer_for_add.text(&start, &end, false).to_string().trim().to_string();
            let link = link_entry_for_resp.text().to_string().trim().to_string();
            let documents = docs_for_add.borrow().join("\\n");
            let tags = tag_entry_for_resp.text().to_string().trim().to_string();
            let due_date = due_date_entry_for_resp.text().to_string().trim().to_string();
            let priority = priority_dropdown_for_resp.selected() as i32;
            if let Some(project_id) = *s.current_project_id.borrow() {
                let db = s.db.borrow();
                let _ = db.create_task(project_id, column_id, &title, &description, &documents, &link, &tags, &due_date, priority);
                drop(db);
                refresh_tasks(&s);
            }
        }
        dialog_for_add.close();
    });

    card.append(&buttons);
    content.append(&card);

    let dialog_activate = add_btn.clone();
    entry.connect_activate(move |_| {
        dialog_activate.emit_clicked();
    });

    dialog.present();
}

pub fn show_edit_task_dialog(state: &Rc<AppState>, task_id: i64) {
    let task = state.db.borrow().get_task(task_id).ok();
    if task.is_none() {
        return;
    }
    let task = task.unwrap();

    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Edit Task"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(400, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    dialog.add_css_class("dialog-window");

    let content = dialog.content_area();
    content.add_css_class("dialog-content");

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.set_margin_top(6);
    card.set_margin_bottom(20);
    card.set_margin_start(20);
    card.set_margin_end(20);

    let title_label = gtk::Label::new(Some("Edit Task"));
    title_label.add_css_class("dialog-title");
    title_label.set_halign(gtk::Align::Start);
    card.append(&title_label);

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Task title"));
    entry.add_css_class("dialog-entry");
    entry.set_text(&task.title);
    card.append(&entry);

    let desc_scroll = gtk::ScrolledWindow::new();
    desc_scroll.set_min_content_height(80);
    desc_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    desc_scroll.add_css_class("dialog-entry");
    let desc_buffer = gtk::TextBuffer::new(None);
    desc_buffer.set_text(&task.description);
    let desc_view = gtk::TextView::with_buffer(&desc_buffer);
    desc_view.set_wrap_mode(gtk::WrapMode::Word);
    desc_view.set_top_margin(6);
    desc_view.set_left_margin(10);
    desc_view.set_right_margin(10);
    desc_view.add_css_class("dialog-textview");
    desc_view.set_hexpand(true);
    desc_view.set_vexpand(true);
    desc_scroll.set_child(Some(&desc_view));
    card.append(&desc_scroll);

    let link_entry = gtk::Entry::new();
    link_entry.set_placeholder_text(Some("Link URL (optional)"));
    link_entry.add_css_class("dialog-entry");
    link_entry.set_text(&task.link);
    card.append(&link_entry);

    let tag_entry = gtk::Entry::new();
    tag_entry.set_placeholder_text(Some("Tags (comma separated, type to see existing)"));
    tag_entry.add_css_class("dialog-entry");
    tag_entry.set_text(&task.tags);
    card.append(&tag_entry);

    let due_date_entry = gtk::Entry::new();
    due_date_entry.set_placeholder_text(Some("Due date (YYYY-MM-DD)"));
    due_date_entry.add_css_class("dialog-entry");
    due_date_entry.set_text(&task.due_date);
    card.append(&due_date_entry);

    let priority_store = gtk::StringList::new(&["Normal", "Low", "Medium", "High", "Critical"]);
    let priority_dropdown =
        gtk::DropDown::new(Some(priority_store.upcast::<gio::ListModel>()), Option::<&gtk::Expression>::None);
    priority_dropdown.add_css_class("dialog-entry");
    priority_dropdown.set_selected(task.priority as u32);
    card.append(&priority_dropdown);

    let completion = gtk::EntryCompletion::new();
    let tag_model = gtk::ListStore::new(&[glib::Type::STRING]);
    completion.set_model(Some(&tag_model));
    completion.set_text_column(0);
    tag_entry.set_completion(Some(&completion));

    if let Some(project_id) = *state.current_project_id.borrow() {
        if let Ok(existing) = state.db.borrow().get_existing_tags(project_id) {
            for tag in &existing {
                tag_model.insert_with_values(None, &[(0, &tag as &dyn ToValue)]);
            }
        }
    }

    let doc_count = if task.documents.is_empty() {
        0
    } else {
        task.documents.lines().count()
    };
    let docs_label_text = if doc_count > 0 {
        format!("{} document{} attached", doc_count, if doc_count == 1 { "" } else { "s" })
    } else {
        "No documents attached".to_string()
    };
    let docs_label = gtk::Label::new(Some(&docs_label_text));
    docs_label.add_css_class("dialog-docs-label");
    docs_label.set_halign(gtk::Align::Start);
    card.append(&docs_label);

    let docs = Rc::new(RefCell::new(
        if task.documents.is_empty() {
            Vec::new()
        } else {
            task.documents.lines().map(|l| l.to_string()).collect()
        }
    ));

    let drop_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
    drop_area.set_size_request(-1, 60);
    drop_area.add_css_class("dialog-drop-area");
    let drop_hint = gtk::Label::new(Some("Drop files here"));
    drop_area.append(&drop_hint);
    card.append(&drop_area);

    let docs_for_drop = docs.clone();
    let docs_label_for_drop = docs_label.clone();
    let formats = gdk::ContentFormats::new(&["text/uri-list"]);
    let drop_target = gtk::DropTarget::builder()
        .formats(&formats)
        .actions(gdk::DragAction::COPY)
        .build();
    drop_target.connect_drop(move |_target, value, _x, _y| {
        if let Ok(uris) = value.get::<String>() {
            let paths: Vec<String> = uris
                .split(|c: char| c == '\n' || c == '\r')
                .filter(|s| !s.is_empty())
                .filter(|u| u.starts_with("file://"))
                .map(|u| u.trim_start_matches("file://").to_string())
                .collect();
            if !paths.is_empty() {
                let mut docs = docs_for_drop.borrow_mut();
                docs.extend(paths);
                let count = docs.len();
                docs_label_for_drop.set_text(&format!("{} document{} attached", count, if count == 1 { "" } else { "s" }));
            }
        }
        true
    });
    drop_area.add_controller(drop_target);

    let (cancel_btn, save_btn, buttons) = make_buttons("Cancel", "Save");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_save = dialog.clone();
    let entry_for_resp = entry.clone();
    let link_entry_for_resp = link_entry.clone();
    let tag_entry_for_resp = tag_entry.clone();
    let docs_for_save = docs.clone();
    let desc_buffer_for_save = desc_buffer.clone();
    let due_date_entry_for_resp = due_date_entry.clone();
    let priority_dropdown_for_resp = priority_dropdown.clone();
    save_btn.connect_clicked(move |_| {
        let title = entry_for_resp.text().to_string().trim().to_string();
        if !title.is_empty() {
            let start = desc_buffer_for_save.start_iter();
            let end = desc_buffer_for_save.end_iter();
            let description = desc_buffer_for_save.text(&start, &end, false).to_string().trim().to_string();
            let link = link_entry_for_resp.text().to_string().trim().to_string();
            let documents = docs_for_save.borrow().join("\n");
            let tags = tag_entry_for_resp.text().to_string().trim().to_string();
            let due_date = due_date_entry_for_resp.text().to_string().trim().to_string();
            let priority = priority_dropdown_for_resp.selected() as i32;
            let db = s.db.borrow();
            let _ = db.update_task(task_id, &title, &description, &documents, &link, &tags, &due_date, priority);
            drop(db);
            refresh_tasks(&s);
            dialog_for_save.close();
        }
    });

    card.append(&buttons);
    content.append(&card);

    let dialog_activate = save_btn.clone();
    entry.connect_activate(move |_| {
        dialog_activate.emit_clicked();
    });

    dialog.present();
}

pub fn show_add_column_dialog(state: &Rc<AppState>) {
    if state.current_project_id.borrow().is_none() {
        return;
    }

    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("New Column"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(320, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));

    let content = dialog.content_area();
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);

    card.append(&gtk::Label::new(Some("New Column")));

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("e.g. Backlog, Review, Deployed"));
    card.append(&entry);

    let (cancel_btn, add_btn, buttons) = make_buttons("Cancel", "Add Column");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_add = dialog.clone();
    let entry_for_resp = entry.clone();
    add_btn.connect_clicked(move |_| {
        let name = entry_for_resp.text().to_string().trim().to_string();
        if !name.is_empty() {
            if let Some(project_id) = *s.current_project_id.borrow() {
                let db = s.db.borrow();
                let _ = db.create_column(project_id, &name);
                drop(db);
                rebuild_columns(&s);
                refresh_tasks(&s);
            }
        }
        dialog_for_add.close();
    });

    card.append(&buttons);
    content.append(&card);

    let dialog_activate = dialog.clone();
    entry.connect_activate(move |_| {
        dialog_activate.response(gtk::ResponseType::Accept);
    });

    dialog.present();
}

pub fn show_delete_column_dialog(state: &Rc<AppState>, column_id: i64, column_name: &str) {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Delete Column"));
    dialog.set_transient_for(Some(&state.window));
    dialog.set_modal(true);
    dialog.set_default_size(320, -1);
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));

    let content = dialog.content_area();
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);

    card.append(&gtk::Label::new(Some("Delete Column")));

    let column_name = column_name.to_string();
    let label = gtk::Label::new(Some(&format!(
        "Delete \"{}\" and all its tasks?",
        column_name
    )));
    label.set_xalign(0.0);
    label.set_wrap(true);
    card.append(&label);

    let (cancel_btn, delete_btn, buttons) = make_buttons("Cancel", "Delete");

    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let s = state.clone();
    let dialog_for_delete = dialog.clone();
    delete_btn.connect_clicked(move |_| {
        let db = s.db.borrow();
        let _ = db.delete_column(column_id);
        drop(db);
        rebuild_columns(&s);
        refresh_tasks(&s);
        dialog_for_delete.close();
    });

    card.append(&buttons);
    content.append(&card);

    dialog.present();
}
