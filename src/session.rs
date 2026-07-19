use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct SessionState {
    pub last_project_id: Option<i64>,
    pub filter_year: Option<i32>,
    pub filter_month: Option<i32>,
}

fn session_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".local/share/ctrl-project-management");
    let _ = fs::create_dir_all(&path);
    path.push("session.json");
    path
}

pub fn save_session(state: &SessionState) {
    let path = session_path();
    if let Ok(json) = serde_json::to_string(state) {
        let _ = fs::write(&path, &json);
    }
}

pub fn load_session() -> SessionState {
    let path = session_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
