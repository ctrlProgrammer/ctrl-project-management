use crate::db::Task;
use crate::dialogs::show_edit_task_dialog;
use crate::models::AppState;
use gtk4::gdk;
use gtk4::gdk::prelude::*;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::rc::Rc;

pub fn refresh_tasks(state: &Rc<AppState>) {
    for cw in state.column_widgets.borrow().iter() {
        while let Some(child) = cw.list.first_child() {
            cw.list.remove(&child);
        }
    }

    if let Some(project_id) = *state.current_project_id.borrow() {
        let db = state.db.borrow();
        let year = *state.filter_year.borrow();
        let month = *state.filter_month.borrow();
        let search = state.filter_search.borrow().to_lowercase();
        let search_empty = search.is_empty();
        let tasks = if year > 0 {
            db.get_tasks_for_project_and_month(project_id, year, month)
        } else {
            db.get_tasks_for_project(project_id)
        };
        if let Ok(tasks) = tasks {
            let column_widgets = state.column_widgets.borrow();
            for task in tasks {
                if !search_empty {
                    let title_match = task.title.to_lowercase().contains(&search);
                    let tags_match = task.tags.to_lowercase().contains(&search);
                    if !title_match && !tags_match {
                        continue;
                    }
                }
                let card = create_task_card(state, &task);
                if let Some(cw) = column_widgets.iter().find(|cw| cw.id == task.column_id) {
                    cw.list.append(&card);
                }
            }
        }
    }

    for cw in state.column_widgets.borrow().iter() {
        let mut count = 0usize;
        let mut iter = cw.list.first_child();
        while let Some(child) = iter {
            count += 1;
            iter = child.next_sibling();
        }
        cw.count
            .set_text(&format!("{} task{}", count, if count == 1 { "" } else { "s" }));
    }
}

pub fn create_task_card(state: &Rc<AppState>, task: &Task) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 4);
    card.add_css_class("task-card");

    let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title = gtk::Label::new(Some(&task.title));
    title.add_css_class("task-title");
    title.set_hexpand(true);
    title.set_xalign(0.0);
    title.set_wrap(true);

    let edit_btn = gtk::Button::from_icon_name("document-edit-symbolic");
    edit_btn.add_css_class("task-edit");
    edit_btn.set_has_frame(false);

    let delete_btn = gtk::Button::with_label("\u{00d7}");
    delete_btn.add_css_class("task-delete");
    delete_btn.set_has_frame(false);

    top_row.append(&title);
    top_row.append(&edit_btn);
    top_row.append(&delete_btn);
    card.append(&top_row);

    if !task.description.is_empty() {
        let first_line = task.description.lines().next().unwrap_or("");
        let preview = if first_line.len() > 80 {
            format!("{}...", &first_line[..80])
        } else {
            first_line.to_string()
        };
        let desc = gtk::Label::new(Some(&preview));
        desc.add_css_class("task-desc");
        desc.set_xalign(0.0);
        desc.set_wrap(true);
        desc.set_lines(2);
        card.append(&desc);
    }

    if !task.link.is_empty() {
        let link_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let link_icon = gtk::Label::new(Some("\u{1f517}"));
        link_icon.add_css_class("task-link-icon");
        let link_label = gtk::Label::new(Some(&task.link));
        link_label.add_css_class("task-link");
        link_label.set_xalign(0.0);
        link_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        link_label.set_max_width_chars(30);
        link_box.append(&link_icon);
        link_box.append(&link_label);
        card.append(&link_box);
    }

    if !task.documents.is_empty() {
        let doc_count = task.documents.lines().count();
        let doc_label = gtk::Label::new(Some(&format!("\u{1f4ce} {} document{}", doc_count, if doc_count == 1 { "" } else { "s" })));
        doc_label.add_css_class("task-docs");
        doc_label.set_xalign(0.0);
        card.append(&doc_label);
    }

    if !task.tags.is_empty() {
        let tag_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tag_box.set_halign(gtk::Align::Start);
        for tag in task.tags.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let tag_label = gtk::Label::new(Some(tag));
            tag_label.add_css_class("task-tag");
            tag_box.append(&tag_label);
        }
        card.append(&tag_box);
    }

    let task_id = task.id;

    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gdk::DragAction::MOVE);
    let id_str = task_id.to_string();
    drag_source.connect_prepare(move |_src, _x, _y| {
        let v = id_str.to_value();
        Some(gdk::ContentProvider::for_value(&v))
    });
    card.add_controller(drag_source);

    let s = state.clone();
    delete_btn.connect_clicked(move |_| {
        let db = s.db.borrow();
        let _ = db.delete_task(task_id);
        drop(db);
        refresh_tasks(&s);
    });

    let s = state.clone();
    edit_btn.connect_clicked(move |_| {
        show_edit_task_dialog(&s, task_id);
    });

    card
}
