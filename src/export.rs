use crate::db::{Database, Task};

fn priority_label(p: i32) -> &'static str {
    match p {
        0 => "Normal",
        1 => "Low",
        2 => "Medium",
        3 => "High",
        4 => "Critical",
        _ => "Unknown",
    }
}

fn priority_badge(p: i32) -> &'static str {
    match p {
        0 => "[Normal]",
        1 => "[Low]",
        2 => "[Medium]",
        3 => "[High]",
        4 => "[Critical]",
        _ => "[?]",
    }
}

#[derive(serde::Serialize)]
struct ExportColumn {
    id: i64,
    name: String,
    position: i32,
    tasks: Vec<ExportTask>,
}

#[derive(serde::Serialize)]
struct ExportTask {
    id: i64,
    title: String,
    description: String,
    documents: String,
    link: String,
    tags: String,
    position: i32,
    created_at: String,
    due_date: String,
    priority: i32,
    priority_label: String,
}

#[derive(serde::Serialize)]
struct ExportProjectJson {
    project: ProjectHeader,
    columns: Vec<ExportColumn>,
}

#[derive(serde::Serialize)]
struct ProjectHeader {
    id: i64,
    name: String,
    description: String,
}

/// Export project data as a JSON string.
/// Format: { "project": { id, name, description }, "columns": [ { id, name, position, tasks: [...] } ] }
pub fn export_project_json(db: &Database, project_id: i64) -> Result<String, String> {
    let projects = db
        .get_all_projects()
        .map_err(|e| format!("Database error: {}", e))?;

    let project_info = projects
        .iter()
        .find(|(id, _, _)| *id == project_id)
        .map(|(id, name, desc)| ProjectHeader {
            id: *id,
            name: name.clone(),
            description: desc.clone(),
        })
        .ok_or_else(|| format!("Project {} not found", project_id))?;

    let columns = db
        .get_columns_for_project(project_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let tasks = db
        .get_tasks_for_project(project_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let export_columns: Vec<ExportColumn> = columns
        .iter()
        .map(|col| {
            let col_tasks: Vec<ExportTask> = tasks
                .iter()
                .filter(|t| t.column_id == col.id)
                .map(|t| ExportTask {
                    id: t.id,
                    title: t.title.clone(),
                    description: t.description.clone(),
                    documents: t.documents.clone(),
                    link: t.link.clone(),
                    tags: t.tags.clone(),
                    position: t.position,
                    created_at: t.created_at.clone(),
                    due_date: t.due_date.clone(),
                    priority: t.priority,
                    priority_label: priority_label(t.priority).to_string(),
                })
                .collect();
            ExportColumn {
                id: col.id,
                name: col.name.clone(),
                position: col.position,
                tasks: col_tasks,
            }
        })
        .collect();

    let export = ExportProjectJson {
        project: project_info,
        columns: export_columns,
    };

    serde_json::to_string_pretty(&export).map_err(|e| format!("Serialization error: {}", e))
}

/// Export project data as a human-readable Markdown string.
pub fn export_project_markdown(db: &Database, project_id: i64) -> Result<String, String> {
    let projects = db
        .get_all_projects()
        .map_err(|e| format!("Database error: {}", e))?;

    let (pid, pname, pdesc) = projects
        .iter()
        .find(|(id, _, _)| *id == project_id)
        .map(|(id, name, desc)| (*id, name.clone(), desc.clone()))
        .ok_or_else(|| format!("Project {} not found", project_id))?;

    let columns = db
        .get_columns_for_project(project_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let tasks = db
        .get_tasks_for_project(project_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let mut md = String::new();

    // Title
    md.push_str(&format!("# {}\n\n", pname));
    if !pdesc.is_empty() {
        md.push_str(&format!("{}\n\n", pdesc));
    }
    md.push_str(&format!("_Project ID: {}_\n\n", pid));
    md.push_str("---\n\n");

    for col in &columns {
        md.push_str(&format!("## {}\n\n", col.name));

        let col_tasks: Vec<&Task> = tasks.iter().filter(|t| t.column_id == col.id).collect();

        if col_tasks.is_empty() {
            md.push_str("*No tasks.*\n\n");
            continue;
        }

        for task in &col_tasks {
            md.push_str(&format!("- **{}**", task.title));

            // Priority badge
            let p_badge = priority_badge(task.priority);
            md.push_str(&format!(" {}", p_badge));

            // Due date
            if !task.due_date.is_empty() {
                md.push_str(&format!(" (Due: {})", task.due_date));
            }

            md.push('\n');

            // Tags
            if !task.tags.is_empty() {
                let tags: Vec<&str> = task.tags.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
                if !tags.is_empty() {
                    let tag_str = tags
                        .iter()
                        .map(|t| format!("`{}`", t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    md.push_str(&format!("  Tags: {}\n", tag_str));
                }
            }

            // Description (only first line or first 100 chars)
            if !task.description.is_empty() {
                let desc = if task.description.len() > 120 {
                    format!("{}…", &task.description[..120])
                } else {
                    task.description.clone()
                };
                let first_line = desc.lines().next().unwrap_or(&desc);
                md.push_str(&format!("  > {}\n", first_line));
            }

            // Link
            if !task.link.is_empty() {
                md.push_str(&format!("  Link: {}\n", task.link));
            }

            md.push('\n');
        }
    }

    md.push_str("---\n");
    md.push_str(&format!(
        "_Exported from ctrl-project-management on {}_\n",
        chrono_now()
    ));

    Ok(md)
}

fn chrono_now() -> String {
    // Use a simple timestamp without pulling in chrono crate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // crude UTC date-time from epoch seconds
    format_datetime(secs)
}

fn format_datetime(epoch_secs: u64) -> String {
    // Simple leap-year-aware date calculation
    let days = epoch_secs / 86400;
    let time_secs = epoch_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining = days as i64;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1usize;
    for &md in month_days.iter() {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hours, minutes, seconds
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
