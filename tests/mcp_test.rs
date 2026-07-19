use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::path::Path;

/// Wraps an MCP server subprocess for testing.
struct McpServer {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// Find the binary path relative to the test runner's exe.
/// The test binary lives at target/debug/deps/mcp_test-<hash>,
/// and the main binary sits at target/debug/ctrl-project-management.
fn binary_path() -> std::path::PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    // Walk up: deps/ -> debug/ -> target/
    let debug_dir = exe
        .parent()
        .expect("exe parent")
        .parent()
        .expect("deps parent");
    debug_dir.join("ctrl-project-management")
}

impl McpServer {
    fn new(db_path: &Path) -> Self {
        let bin = binary_path();
        let mut child = Command::new(&bin)
            .arg("mcp")
            .arg(db_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn MCP server");

        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));

        McpServer { child, stdin, stdout }
    }

    fn send(&mut self, request: &serde_json::Value) {
        let line = serde_json::to_string(request).unwrap();
        writeln!(self.stdin, "{}", line).unwrap();
        self.stdin.flush().unwrap();
    }

    fn recv(&mut self) -> serde_json::Value {
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        serde_json::from_str(line.trim()).unwrap()
    }

    fn request(
        &mut self,
        method: &str,
        id: i64,
        params: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.send(&req);
        self.recv()
    }
}

impl Drop for McpServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_mcp_full_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut srv = McpServer::new(&db_path);

    // ── 1. initialize ──────────────────────────────────────────
    let resp = srv.request("initialize", 1, None);
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(resp["result"]["serverInfo"]["name"], "ctrl-project-management");

    // ── 2. tools/list ──────────────────────────────────────────
    let resp = srv.request("tools/list", 2, None);
    assert_eq!(resp["id"], 2);
    let tools = resp["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for name in &[
        "list_projects",
        "get_columns",
        "list_tasks",
        "get_task",
        "create_task",
        "update_task",
        "delete_task",
        "create_column",
        "delete_column",
        "create_project",
        "delete_project",
    ] {
        assert!(tool_names.contains(name), "missing tool: {}", name);
    }

    // ── 3. tools/call list_projects (empty) ────────────────────
    let resp = srv.request(
        "tools/call",
        3,
        Some(serde_json::json!({"name": "list_projects", "arguments": {}})),
    );
    assert_eq!(resp["id"], 3);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let projects: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
    assert_eq!(projects.len(), 0);

    // ── 4. tools/call create_project ───────────────────────────
    let resp = srv.request(
        "tools/call",
        4,
        Some(serde_json::json!({
            "name": "create_project",
            "arguments": {"name": "Test Project", "description": "A test"}
        })),
    );
    assert_eq!(resp["id"], 4);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let project: serde_json::Value = serde_json::from_str(text).unwrap();
    let project_id = project["id"].as_i64().unwrap();
    assert_eq!(project["name"], "Test Project");

    // ── 5. tools/call get_columns ──────────────────────────────
    let resp = srv.request(
        "tools/call",
        5,
        Some(serde_json::json!({
            "name": "get_columns",
            "arguments": {"project_id": project_id}
        })),
    );
    assert_eq!(resp["id"], 5);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let columns: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0]["name"], "To Do");
    assert_eq!(columns[1]["name"], "In Progress");
    assert_eq!(columns[2]["name"], "Done");
    let todo_col_id = columns[0]["id"].as_i64().unwrap();

    // ── 6. tools/call create_task ──────────────────────────────
    let resp = srv.request(
        "tools/call",
        6,
        Some(serde_json::json!({
            "name": "create_task",
            "arguments": {
                "project_id": project_id,
                "column_id": todo_col_id,
                "title": "Test Task",
                "description": "A test task",
                "tags": "tag1,tag2",
                "priority": 2
            }
        })),
    );
    assert_eq!(resp["id"], 6);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let task: serde_json::Value = serde_json::from_str(text).unwrap();
    let task_id = task["id"].as_i64().unwrap();
    assert_eq!(task["title"], "Test Task");
    assert_eq!(task["column_id"], todo_col_id);

    // ── 7. tools/call list_tasks ───────────────────────────────
    let resp = srv.request(
        "tools/call",
        7,
        Some(serde_json::json!({
            "name": "list_tasks",
            "arguments": {"project_id": project_id}
        })),
    );
    assert_eq!(resp["id"], 7);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let tasks: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Test Task");

    // ── 8. tools/call update_task ──────────────────────────────
    let resp = srv.request(
        "tools/call",
        8,
        Some(serde_json::json!({
            "name": "update_task",
            "arguments": {
                "task_id": task_id,
                "title": "Updated Task",
                "priority": 3
            }
        })),
    );
    assert_eq!(resp["id"], 8);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let updated: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(updated["title"], "Updated Task");
    assert_eq!(updated["priority"], 3);

    // ── 9. tools/call delete_task ──────────────────────────────
    let resp = srv.request(
        "tools/call",
        9,
        Some(serde_json::json!({
            "name": "delete_task",
            "arguments": {"task_id": task_id}
        })),
    );
    assert_eq!(resp["id"], 9);
    assert!(resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("deleted"));

    // ── 10. tools/call create_column ───────────────────────────
    let resp = srv.request(
        "tools/call",
        10,
        Some(serde_json::json!({
            "name": "create_column",
            "arguments": {"project_id": project_id, "name": "Review"}
        })),
    );
    assert_eq!(resp["id"], 10);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let col: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(col["name"], "Review");
    let col_id = col["id"].as_i64().unwrap();

    // ── 11. tools/call delete_column ───────────────────────────
    let resp = srv.request(
        "tools/call",
        11,
        Some(serde_json::json!({
            "name": "delete_column",
            "arguments": {"column_id": col_id}
        })),
    );
    assert_eq!(resp["id"], 11);
    assert!(resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("deleted"));

    // ── 12. tools/call delete_project ──────────────────────────
    let resp = srv.request(
        "tools/call",
        12,
        Some(serde_json::json!({
            "name": "delete_project",
            "arguments": {"project_id": project_id}
        })),
    );
    assert_eq!(resp["id"], 12);
    assert!(resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("deleted"));

    // ── 13. Error: unknown tool ────────────────────────────────
    let resp = srv.request(
        "tools/call",
        13,
        Some(serde_json::json!({
            "name": "nonexistent_tool",
            "arguments": {}
        })),
    );
    assert_eq!(resp["error"]["code"], -32601);
    assert!(resp["error"]["message"].as_str().unwrap().contains("Method not found"));

    // ── 14. Error: unknown method ──────────────────────────────
    let resp = srv.request("bogus_method", 14, None);
    assert_eq!(resp["error"]["code"], -32601);

    // ── 15. Error: missing required params ─────────────────────
    let resp = srv.request(
        "tools/call",
        15,
        Some(serde_json::json!({
            "name": "get_columns",
            "arguments": {}
        })),
    );
    assert_eq!(resp["error"]["code"], -32602);
    assert!(resp["error"]["message"].as_str().unwrap().contains("Invalid params"));
}
