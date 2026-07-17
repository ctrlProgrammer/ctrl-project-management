use crate::db::Database;
use gtk4::{self as gtk};
use std::cell::RefCell;

pub struct ProjectInfo {
    pub id: i64,
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
}

pub struct ColumnWidgets {
    pub id: i64,
    pub list: gtk::ListBox,
    pub count: gtk::Label,
    pub column_box: gtk::Box,
}

pub struct AppState {
    pub db: RefCell<Database>,
    pub current_project_id: RefCell<Option<i64>>,
    pub projects: RefCell<Vec<ProjectInfo>>,
    pub column_widgets: RefCell<Vec<ColumnWidgets>>,
    pub project_selector: gtk::MenuButton,
    pub project_listbox: gtk::ListBox,
    pub project_search: gtk::Entry,
    pub kanban_box: gtk::Box,
    pub add_column_btn: gtk::Button,
    pub filter_year: RefCell<i32>,
    pub filter_month: RefCell<i32>,
    pub filter_label: gtk::Label,
    pub filter_search: RefCell<String>,
    pub task_search: gtk::SearchEntry,
    pub window: gtk::ApplicationWindow,
}
