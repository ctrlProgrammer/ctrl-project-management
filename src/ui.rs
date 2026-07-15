use crate::columns::rebuild_columns;
use crate::css::APP_CSS;
use crate::dialogs::{show_add_column_dialog, show_delete_project_dialog, show_new_project_dialog};
use crate::models::AppState;
use crate::projects::refresh_projects;
use crate::tasks::refresh_tasks;
use gtk4::gdk;
use gtk4::gdk::prelude::ToplevelExt;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_ui(app: &gtk::Application) {
    let database = crate::db::Database::open(crate::db::default_db_path()).expect("Failed to open database");

    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Project Manager"));
    window.set_decorated(false);
    window.set_default_size(1060, 640);

    let display = gtk4::gdk::Display::default().expect("No display");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(APP_CSS);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&main_box));

    let widget_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    widget_box.add_css_class("widget-container");
    main_box.append(&widget_box);

    let drag_handle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    drag_handle.add_css_class("drag-handle");
    widget_box.append(&drag_handle);

    let drag = gtk::GestureDrag::new();
    let window_for_drag = window.clone();
    drag.connect_drag_begin(move |_gesture, start_x, start_y| {
        if let Some(surface) = window_for_drag.surface() {
            if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
                if let Some(seat) = gdk::Display::default().and_then(|d| d.default_seat()) {
                    if let Some(pointer) = seat.pointer() {
                        toplevel.begin_move(&pointer, 1, start_x, start_y, 0);
                    }
                }
            }
        }
    });
    drag_handle.add_controller(drag);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("header-box");

    let project_label = gtk::Label::new(Some("Project:"));
    project_label.add_css_class("project-label");

    let project_selector = gtk::MenuButton::new();
    project_selector.add_css_class("project-selector");
    project_selector.set_label("Select a project...");
    project_selector.set_has_frame(false);

    let popover = gtk::Popover::new();
    popover.set_has_arrow(false);
    popover.add_css_class("project-popover");
    project_selector.set_popover(Some(&popover));

    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    popover_box.set_size_request(260, -1);

    let project_search = gtk::Entry::new();
    project_search.set_placeholder_text(Some("Search projects..."));
    project_search.add_css_class("project-search");

    let project_listbox = gtk::ListBox::new();
    project_listbox.set_activate_on_single_click(true);
    project_listbox.add_css_class("project-list");
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&project_listbox));
    scroll.set_min_content_height(180);
    scroll.set_max_content_height(300);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    popover_box.append(&project_search);
    popover_box.append(&scroll);
    popover.set_child(Some(&popover_box));

    let new_project_btn = gtk::Button::with_label("+ New Project");

    let delete_project_btn = gtk::Button::with_label("Delete Project");
    delete_project_btn.add_css_class("column-delete");
    delete_project_btn.add_css_class("delete-project-btn");

    header.append(&project_label);
    header.append(&project_selector);

    let prev_month_btn = gtk::Button::with_label("\u{2039}");
    prev_month_btn.add_css_class("month-nav-btn");
    prev_month_btn.set_has_frame(false);

    let filter_label = gtk::Label::new(None);
    filter_label.add_css_class("filter-label");

    let next_month_btn = gtk::Button::with_label("\u{203a}");
    next_month_btn.add_css_class("month-nav-btn");
    next_month_btn.set_has_frame(false);

    let current_btn = gtk::Button::with_label("Current");
    current_btn.add_css_class("current-month-btn");
    current_btn.set_has_frame(false);

    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    filter_box.add_css_class("filter-box");
    filter_box.set_homogeneous(false);
    filter_box.append(&prev_month_btn);
    filter_box.append(&filter_label);
    filter_box.append(&next_month_btn);
    filter_box.append(&current_btn);

    let header_fill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_fill.set_hexpand(true);
    header.append(&header_fill);
    header.append(&filter_box);
    header.append(&delete_project_btn);
    header.append(&new_project_btn);

    widget_box.append(&header);

    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    widget_box.append(&sep);

    let kanban_scroll = gtk::ScrolledWindow::new();
    kanban_scroll.set_hexpand(true);
    kanban_scroll.set_vexpand(true);
    kanban_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);

    let kanban_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    kanban_box.set_hexpand(true);
    kanban_box.set_vexpand(true);
    kanban_scroll.set_child(Some(&kanban_box));
    widget_box.append(&kanban_scroll);

    let add_column_btn = gtk::Button::with_label("+ Add Column");
    add_column_btn.add_css_class("add-column-outer");

    let now = glib::DateTime::now_local().unwrap();
    let current_year = now.year();
    let current_month = now.month();

    filter_label.set_text(&month_name(current_month, current_year));

    let state = Rc::new(AppState {
        db: RefCell::new(database),
        current_project_id: RefCell::new(None),
        projects: RefCell::new(Vec::new()),
        column_widgets: RefCell::new(Vec::new()),
        project_selector: project_selector.clone(),
        project_listbox: project_listbox.clone(),
        project_search: project_search.clone(),
        kanban_box: kanban_box.clone(),
        add_column_btn: add_column_btn.clone(),
        filter_year: RefCell::new(current_year),
        filter_month: RefCell::new(current_month),
        filter_label: filter_label.clone(),
        window: window.clone(),
    });

    let s = state.clone();
    project_search.connect_changed(move |entry| {
        let text = entry.text().to_lowercase();
        let listbox = &s.project_listbox;
        let mut child = listbox.first_child();
        while let Some(widget) = child {
            if let Some(row) = widget.downcast_ref::<gtk::ListBoxRow>() {
                let should_show = if let Some(child_widget) = row.child() {
                    if let Some(label) = child_widget.downcast_ref::<gtk::Label>() {
                        text.is_empty() || label.text().to_lowercase().contains(&text)
                    } else {
                        true
                    }
                } else {
                    true
                };
                row.set_visible(should_show);
            }
            child = widget.next_sibling();
        }
    });

    let s = state.clone();
    project_listbox.connect_row_activated(move |_listbox, row| {
        let index = row.index();
        let projects = s.projects.borrow();
        if let Some(project) = projects.get(index as usize) {
            *s.current_project_id.borrow_mut() = Some(project.id);
            s.project_selector.set_label(&project.name);
            s.project_search.set_text("");
            if let Some(popover) = s.project_selector.popover() {
                popover.popdown();
            }
            rebuild_columns(&s);
            refresh_tasks(&s);
        }
    });

    let s = state.clone();
    new_project_btn.connect_clicked(move |_| {
        show_new_project_dialog(&s);
    });

    let s = state.clone();
    delete_project_btn.connect_clicked(move |_| {
        show_delete_project_dialog(&s);
    });

    let s = state.clone();
    add_column_btn.connect_clicked(move |_| {
        show_add_column_dialog(&s);
    });

    let s = state.clone();
    prev_month_btn.connect_clicked(move |_| {
        let new_month;
        let new_year;
        {
            let mut year = s.filter_year.borrow_mut();
            let mut month = s.filter_month.borrow_mut();
            if *month == 1 {
                *month = 12;
                *year -= 1;
            } else {
                *month -= 1;
            }
            new_month = *month;
            new_year = *year;
        }
        s.filter_label.set_text(&month_name(new_month, new_year));
        refresh_tasks(&s);
    });

    let s = state.clone();
    next_month_btn.connect_clicked(move |_| {
        let new_month;
        let new_year;
        {
            let mut year = s.filter_year.borrow_mut();
            let mut month = s.filter_month.borrow_mut();
            if *month == 12 {
                *month = 1;
                *year += 1;
            } else {
                *month += 1;
            }
            new_month = *month;
            new_year = *year;
        }
        s.filter_label.set_text(&month_name(new_month, new_year));
        refresh_tasks(&s);
    });

    let s = state.clone();
    current_btn.connect_clicked(move |_| {
        let now = glib::DateTime::now_local().unwrap();
        let new_year = now.year();
        let new_month = now.month();
        {
            let mut year = s.filter_year.borrow_mut();
            let mut month = s.filter_month.borrow_mut();
            *year = new_year;
            *month = new_month;
        }
        s.filter_label.set_text(&month_name(new_month, new_year));
        refresh_tasks(&s);
    });

    refresh_projects(&state);
    window.present();
}

fn month_name(month: i32, year: i32) -> String {
    let names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let idx = (month - 1).clamp(0, 11) as usize;
    format!("{} {}", names[idx], year)
}
