mod db;
mod models;
mod css;
mod projects;
mod columns;
mod tasks;
mod dialogs;
mod ui;
mod mcp;

use gtk4::prelude::*;
use gtk4::{self as gtk};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("mcp") {
        let db_path = args.get(2).cloned().unwrap_or_else(|| db::default_db_path().to_string_lossy().to_string());
        eprintln!("ctrl-project-management: starting MCP server (db: {})", db_path);
        if let Err(e) = mcp::run(&db_path) {
            eprintln!("ctrl-project-management: MCP server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let app = gtk::Application::new(Some("com.ctrl.projectmanagement"), Default::default());
    app.connect_activate(ui::build_ui);
    app.run();
}
