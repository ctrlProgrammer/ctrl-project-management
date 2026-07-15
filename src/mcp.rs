use crate::db::Database;
use serde::Deserialize;
use std::io::{self, BufRead, Write};

pub fn run() -> Result<(), String> {
    let db = Database::open("kanban.db").map_err(|e| format!("Failed to open database: {}", e))?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.is_empty() => continue,
            Ok(l) => l,
            Err(e) => {
                eprintln!("MCP: Failed to read stdin: {}", e);
                break;
            }
        };

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let id: Option<serde_json::Value> =
                    serde_json::from_str(&line).ok().and_then(|r: JsonRpcRequest| r.id);
                let err = make_error(id, -32700, "Parse error", &e.to_string());
                write_message(&mut stdout, &err);
                continue;
            }
        };

        let response = handle_request(&db, &request);
        write_message(&mut stdout, &response);
    }

    Ok(())
}

fn write_message(stdout: &mut impl Write, msg: &serde_json::Value) {
    let json = serde_json::to_string(msg).unwrap();
    writeln!(stdout, "{}", json).unwrap();
    stdout.flush().unwrap();
}

fn make_error(id: Option<serde_json::Value>, code: i32, message: &str, data: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
            "data": data
        }
    })
}

fn make_result(id: Option<serde_json::Value>, result: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

fn handle_request(db: &Database, req: &JsonRpcRequest) -> serde_json::Value {
    match req.method.as_str() {
        "initialize" => {
            make_result(req.id.clone(), serde_json::json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": { "name": "ctrl-project-management", "version": "0.1.0" },
                "capabilities": { "tools": {} }
            }))
        }
        "tools/list" => {
            make_result(req.id.clone(), serde_json::json!({
                "tools": [
                    {
                        "name": "list_projects",
                        "description": "List all projects in the kanban board",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "get_columns",
                        "description": "Get all columns for a project",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "integer", "description": "ID of the project" }
                            },
                            "required": ["project_id"]
                        }
                    },
                    {
                        "name": "list_tasks",
                        "description": "List tasks for a project, optionally filtered by year/month",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "integer", "description": "ID of the project" },
                                "year": { "type": "integer", "description": "Filter by year (e.g. 2026)" },
                                "month": { "type": "integer", "description": "Filter by month (1-12)" }
                            },
                            "required": ["project_id"]
                        }
                    },
                    {
                        "name": "get_task",
                        "description": "Get a single task by its ID",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "integer", "description": "ID of the task" }
                            },
                            "required": ["task_id"]
                        }
                    },
                    {
                        "name": "create_task",
                        "description": "Create a new task in a column",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "integer", "description": "ID of the project" },
                                "column_id": { "type": "integer", "description": "ID of the column" },
                                "title": { "type": "string", "description": "Title of the task" },
                                "description": { "type": "string", "description": "Description text" },
                                "link": { "type": "string", "description": "URL link" },
                                "tags": { "type": "string", "description": "Comma-separated tags" }
                            },
                            "required": ["project_id", "column_id", "title"]
                        }
                    },
                    {
                        "name": "update_task",
                        "description": "Update an existing task. Only provided fields are changed.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "integer", "description": "ID of the task" },
                                "title": { "type": "string", "description": "New title" },
                                "description": { "type": "string", "description": "New description" },
                                "link": { "type": "string", "description": "New link URL" },
                                "tags": { "type": "string", "description": "New comma-separated tags" },
                                "column_id": { "type": "integer", "description": "Move to a different column" }
                            },
                            "required": ["task_id"]
                        }
                    },
                    {
                        "name": "create_project",
                        "description": "Create a new project with default columns",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Project name" },
                                "description": { "type": "string", "description": "Project description" }
                            },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "create_column",
                        "description": "Add a new column to a project",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "integer", "description": "ID of the project" },
                                "name": { "type": "string", "description": "Column name" }
                            },
                            "required": ["project_id", "name"]
                        }
                    }
                ]
            }))
        }
        "tools/call" => {
            let name = req.params.as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = req.params.as_ref()
                .and_then(|p| p.get("arguments"));

            match name {
                "list_projects" => handle_list_projects(db, req.id.clone()),
                "get_columns" => handle_get_columns(db, req.id.clone(), arguments),
                "list_tasks" => handle_list_tasks(db, req.id.clone(), arguments),
                "get_task" => handle_get_task(db, req.id.clone(), arguments),
                "create_task" => handle_create_task(db, req.id.clone(), arguments),
                "update_task" => handle_update_task(db, req.id.clone(), arguments),
                "create_project" => handle_create_project(db, req.id.clone(), arguments),
                "create_column" => handle_create_column(db, req.id.clone(), arguments),
                _ => make_error(req.id.clone(), -32601, "Method not found", &format!("Unknown tool: {}", name)),
            }
        }
        _ => make_error(req.id.clone(), -32601, "Method not found", &format!("Unknown method: {}", req.method)),
    }
}

fn handle_list_projects(db: &Database, id: Option<serde_json::Value>) -> serde_json::Value {
    match db.get_all_projects() {
        Ok(projects) => {
            let list: Vec<serde_json::Value> = projects
                .iter()
                .map(|(id, name, description)| {
                    serde_json::json!({ "id": id, "name": name, "description": description })
                })
                .collect();
            make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&list).unwrap() }] }))
        }
        Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
    }
}

fn handle_get_columns(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let project_id = args.and_then(|a| a.get("project_id")).and_then(|v| v.as_i64());
    match project_id {
        None => make_error(id, -32602, "Invalid params", "Missing required parameter: project_id"),
        Some(pid) => match db.get_columns_for_project(pid) {
            Ok(columns) => {
                let list: Vec<serde_json::Value> = columns
                    .iter()
                    .map(|c| serde_json::json!({ "id": c.id, "project_id": c.project_id, "name": c.name, "position": c.position }))
                    .collect();
                make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&list).unwrap() }] }))
            }
            Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
        },
    }
}

fn handle_list_tasks(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let project_id = args.and_then(|a| a.get("project_id")).and_then(|v| v.as_i64());
    match project_id {
        None => make_error(id, -32602, "Invalid params", "Missing required parameter: project_id"),
        Some(pid) => {
            let year = args.and_then(|a| a.get("year")).and_then(|v| v.as_i64());
            let month = args.and_then(|a| a.get("month")).and_then(|v| v.as_i64());
            let result = match (year, month) {
                (Some(y), Some(m)) => db.get_tasks_for_project_and_month(pid, y as i32, m as i32),
                _ => db.get_tasks_for_project(pid),
            };
            match result {
                Ok(tasks) => {
                    let list: Vec<serde_json::Value> = tasks
                        .iter()
                        .map(|t| serde_json::json!({
                            "id": t.id,
                            "project_id": pid,
                            "column_id": t.column_id,
                            "title": t.title,
                            "description": t.description,
                            "documents": t.documents,
                            "link": t.link,
                            "tags": t.tags,
                            "position": t.position,
                            "created_at": t.created_at
                        }))
                        .collect();
                    make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&list).unwrap() }] }))
                }
                Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
            }
        }
    }
}

fn handle_get_task(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let task_id = args.and_then(|a| a.get("task_id")).and_then(|v| v.as_i64());
    match task_id {
        None => make_error(id, -32602, "Invalid params", "Missing required parameter: task_id"),
        Some(tid) => match db.get_task(tid) {
            Ok(task) => {
                let obj = serde_json::json!({
                    "id": task.id,
                    "project_id": task.project_id,
                    "column_id": task.column_id,
                    "title": task.title,
                    "description": task.description,
                    "documents": task.documents,
                    "link": task.link,
                    "tags": task.tags,
                    "position": task.position,
                    "created_at": task.created_at
                });
                make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&obj).unwrap() }] }))
            }
            Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
        },
    }
}

fn handle_create_task(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let args = match args {
        Some(a) => a,
        None => return make_error(id, -32602, "Invalid params", "Missing arguments"),
    };
    let project_id = match args.get("project_id").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: project_id"),
    };
    let column_id = match args.get("column_id").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: column_id"),
    };
    let title = match args.get("title").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: title"),
    };
    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let link = args.get("link").and_then(|v| v.as_str()).unwrap_or("");
    let tags = args.get("tags").and_then(|v| v.as_str()).unwrap_or("");

    match db.create_task(project_id, column_id, title, description, "", link, tags) {
        Ok(task_id) => match db.get_task(task_id) {
            Ok(task) => {
                let obj = serde_json::json!({
                    "id": task.id, "column_id": task.column_id, "title": task.title, "created_at": task.created_at
                });
                make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&obj).unwrap() }] }))
            }
            Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
        },
        Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
    }
}

fn handle_update_task(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let args = match args {
        Some(a) => a,
        None => return make_error(id, -32602, "Invalid params", "Missing arguments"),
    };
    let task_id = match args.get("task_id").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: task_id"),
    };
    let existing = match db.get_task(task_id) {
        Ok(t) => t,
        Err(e) => return make_error(id, -32603, "Database error", &e.to_string()),
    };
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or(&existing.title);
    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or(&existing.description);
    let link = args.get("link").and_then(|v| v.as_str()).unwrap_or(&existing.link);
    let tags = args.get("tags").and_then(|v| v.as_str()).unwrap_or(&existing.tags);

    match db.update_task(task_id, title, description, &existing.documents, link, tags) {
        Ok(()) => {
            if let Some(col_id) = args.get("column_id").and_then(|v| v.as_i64()) {
                let _ = db.update_task_column(task_id, col_id);
            }
            match db.get_task(task_id) {
                Ok(task) => {
                    let obj = serde_json::json!({
                        "id": task.id, "column_id": task.column_id, "title": task.title,
                        "description": task.description, "link": task.link, "tags": task.tags,
                        "created_at": task.created_at
                    });
                    make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&obj).unwrap() }] }))
                }
                Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
            }
        }
        Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
    }
}

fn handle_create_project(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let args = match args {
        Some(a) => a,
        None => return make_error(id, -32602, "Invalid params", "Missing arguments"),
    };
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: name"),
    };
    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

    match db.create_project(name, description) {
        Ok(project_id) => {
            make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&serde_json::json!({ "id": project_id, "name": name })).unwrap() }] }))
        }
        Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
    }
}

fn handle_create_column(db: &Database, id: Option<serde_json::Value>, args: Option<&serde_json::Value>) -> serde_json::Value {
    let args = match args {
        Some(a) => a,
        None => return make_error(id, -32602, "Invalid params", "Missing arguments"),
    };
    let project_id = match args.get("project_id").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: project_id"),
    };
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return make_error(id, -32602, "Invalid params", "Missing required parameter: name"),
    };

    match db.create_column(project_id, name) {
        Ok(column_id) => {
            make_result(id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&serde_json::json!({ "id": column_id, "project_id": project_id, "name": name })).unwrap() }] }))
        }
        Err(e) => make_error(id, -32603, "Database error", &e.to_string()),
    }
}
