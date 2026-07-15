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
        eprintln!("ctrl-project-management: starting MCP server");
        if let Err(e) = mcp::run() {
            eprintln!("ctrl-project-management: MCP server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let app = gtk::Application::new(Some("com.ctrl.projectmanagement"), Default::default());
    app.connect_activate(ui::build_ui);
    app.run();
}
