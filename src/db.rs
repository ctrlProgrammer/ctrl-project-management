use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".local/share/ctrl-project-management/kanban.db");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    path
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: String,
    pub documents: String,
    pub link: String,
    pub tags: String,
    pub column_id: i64,
    pub position: i32,
    pub created_at: String,
    pub due_date: String,
    pub priority: i32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Column {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub position: i32,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS columns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );",
        )?;

        if self.column_exists("tasks", "status") {
            for project_id in self
                .conn
                .prepare("SELECT id FROM projects")?
                .query_map([], |row| row.get::<_, i64>(0))?
                .filter_map(|r| r.ok())
            {
                let col_count: i64 = self.conn.query_row(
                    "SELECT COUNT(*) FROM columns WHERE project_id = ?1",
                    params![project_id],
                    |row| row.get(0),
                )?;
                if col_count == 0 {
                    let default_names = ["To Do", "In Progress", "Done"];
                    for (pos, name) in default_names.iter().enumerate() {
                        self.conn.execute(
                            "INSERT INTO columns (project_id, name, position) VALUES (?1, ?2, ?3)",
                            params![project_id, name, pos as i32],
                        )?;
                    }
                }
            }

            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS tasks_v2 (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    column_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    position INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                );",
            )?;

            self.conn.execute(
                "INSERT INTO tasks_v2 (id, project_id, column_id, title, position, created_at)
                 SELECT t.id, t.project_id, c.id, t.title, t.position, t.created_at
                 FROM tasks t
                 JOIN columns c ON c.project_id = t.project_id AND c.position = t.status",
                [],
            )?;

            self.conn.execute_batch(
                "DROP TABLE IF EXISTS tasks_old;
                 ALTER TABLE tasks RENAME TO tasks_old;
                 ALTER TABLE tasks_v2 RENAME TO tasks;
                 DROP TABLE tasks_old;",
            )?;
        }

        if !self.column_exists("projects", "description") {
            self.conn.execute(
                "ALTER TABLE projects ADD COLUMN description TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        if !self.column_exists("tasks", "column_id") {
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    column_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    documents TEXT NOT NULL DEFAULT '',
                    link TEXT NOT NULL DEFAULT '',
                    tags TEXT NOT NULL DEFAULT '',
                    position INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                );",
            )?;
        }

        if !self.column_exists("tasks", "description") {
            self.conn.execute_batch(
                "ALTER TABLE tasks ADD COLUMN description TEXT NOT NULL DEFAULT '';
                 ALTER TABLE tasks ADD COLUMN documents TEXT NOT NULL DEFAULT '';
                 ALTER TABLE tasks ADD COLUMN link TEXT NOT NULL DEFAULT '';"
            )?;
        }

        if !self.column_exists("tasks", "tags") {
            self.conn.execute(
                "ALTER TABLE tasks ADD COLUMN tags TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        if !self.column_exists("tasks", "due_date") {
            self.conn.execute(
                "ALTER TABLE tasks ADD COLUMN due_date TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        if !self.column_exists("tasks", "priority") {
            self.conn.execute(
                "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    fn column_exists(&self, table: &str, column: &str) -> bool {
        let sql = format!("PRAGMA table_info({})", table);
        if let Ok(mut stmt) = self.conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) {
                return rows.filter_map(|r| r.ok()).any(|c| c == column);
            }
        }
        false
    }

    pub fn get_all_projects(&self) -> Result<Vec<(i64, String, String)>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description FROM projects ORDER BY created_at")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        Ok(projects)
    }

    pub fn delete_project(&self, project_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM projects WHERE id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    pub fn create_project(&self, name: &str, description: &str) -> Result<i64, rusqlite::Error> {
        self.conn
            .execute("INSERT INTO projects (name, description) VALUES (?1, ?2)", params![name, description])?;
        let project_id = self.conn.last_insert_rowid();

        let default_names = ["To Do", "In Progress", "Done"];
        for (pos, name) in default_names.iter().enumerate() {
            self.conn.execute(
                "INSERT INTO columns (project_id, name, position) VALUES (?1, ?2, ?3)",
                params![project_id, name, pos as i32],
            )?;
        }

        Ok(project_id)
    }

    pub fn get_columns_for_project(
        &self,
        project_id: i64,
    ) -> Result<Vec<Column>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, position FROM columns WHERE project_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(Column {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                position: row.get(3)?,
            })
        })?;
        let mut columns = Vec::new();
        for row in rows {
            columns.push(row?);
        }
        Ok(columns)
    }

    pub fn create_column(&self, project_id: i64, name: &str) -> Result<i64, rusqlite::Error> {
        let max_pos: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) FROM columns WHERE project_id = ?1",
                params![project_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        self.conn.execute(
            "INSERT INTO columns (project_id, name, position) VALUES (?1, ?2, ?3)",
            params![project_id, name, max_pos + 1],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_column(&self, column_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM tasks WHERE column_id = ?1",
            params![column_id],
        )?;
        self.conn.execute(
            "DELETE FROM columns WHERE id = ?1",
            params![column_id],
        )?;
        Ok(())
    }

    pub fn get_tasks_for_project(&self, project_id: i64) -> Result<Vec<Task>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, documents, link, tags, column_id, position, created_at, due_date, priority FROM tasks WHERE project_id = ?1 ORDER BY position, created_at",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id,
                title: row.get(1)?,
                description: row.get(2)?,
                documents: row.get(3)?,
                link: row.get(4)?,
                tags: row.get(5)?,
                column_id: row.get(6)?,
                position: row.get(7)?,
                created_at: row.get(8)?,
                due_date: row.get(9)?,
                priority: row.get(10)?,
            })
        })?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    pub fn get_tasks_for_project_and_month(
        &self,
        project_id: i64,
        year: i32,
        month: i32,
    ) -> Result<Vec<Task>, rusqlite::Error> {
        let prefix = format!("{:04}-{:02}", year, month);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, documents, link, tags, column_id, position, created_at, due_date, priority
             FROM tasks
             WHERE project_id = ?1 AND substr(created_at, 1, 7) = ?2
             ORDER BY position, created_at",
        )?;
        let rows = stmt.query_map(params![project_id, prefix], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id,
                title: row.get(1)?,
                description: row.get(2)?,
                documents: row.get(3)?,
                link: row.get(4)?,
                tags: row.get(5)?,
                column_id: row.get(6)?,
                position: row.get(7)?,
                created_at: row.get(8)?,
                due_date: row.get(9)?,
                priority: row.get(10)?,
            })
        })?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    pub fn create_task(
        &self,
        project_id: i64,
        column_id: i64,
        title: &str,
        description: &str,
        documents: &str,
        link: &str,
        tags: &str,
        due_date: &str,
        priority: i32,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO tasks (project_id, column_id, title, description, documents, link, tags, due_date, priority) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![project_id, column_id, title, description, documents, link, tags, due_date, priority],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_existing_tags(&self, project_id: i64) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT tags FROM tasks WHERE project_id = ?1 AND tags != ''",
        )?;
        let rows = stmt.query_map(params![project_id], |row| row.get::<_, String>(0))?;
        let mut all = Vec::new();
        for row in rows {
            let tags_str = row?;
            for tag in tags_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
                if !all.contains(&tag.to_string()) {
                    all.push(tag.to_string());
                }
            }
        }
        all.sort();
        Ok(all)
    }

    pub fn update_task_column(
        &self,
        task_id: i64,
        column_id: i64,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tasks SET column_id = ?1 WHERE id = ?2",
            params![column_id, task_id],
        )?;
        Ok(())
    }

    /// Move a task to a column at a specific position, re-ordering siblings to fill the gap.
    /// Handles both intra-column reorder and cross-column moves.
    pub fn move_task(&self, task_id: i64, to_column_id: i64, to_position: i32) -> Result<(), rusqlite::Error> {
        let from_column_id: i64 = self.conn.query_row(
            "SELECT column_id FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )?;

        // Compact source column when moving cross-column
        if from_column_id != to_column_id {
            let from_pos: i32 = self.conn.query_row(
                "SELECT position FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            ).unwrap_or(0);
            self.conn.execute(
                "UPDATE tasks SET position = position - 1 WHERE column_id = ?1 AND position > ?2",
                params![from_column_id, from_pos],
            )?;
        }

        // Get ordered tasks in target column (excluding current if already there)
        let mut stmt = self.conn.prepare(
            "SELECT id FROM tasks WHERE column_id = ?1 AND id != ?2 ORDER BY position, created_at"
        )?;
        let mut ordered: Vec<i64> = stmt.query_map(params![to_column_id, task_id], |row| row.get::<_, i64>(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Clamp and insert at target position
        let pos = to_position.min(ordered.len() as i32);
        ordered.insert(pos as usize, task_id);

        // Re-assign positions sequentially
        for (i, tid) in ordered.iter().enumerate() {
            self.conn.execute(
                "UPDATE tasks SET position = ?1, column_id = ?2 WHERE id = ?3",
                params![i as i32, to_column_id, tid],
            )?;
        }

        Ok(())
    }

    pub fn reorder_column(&self, column_id: i64, new_position: i32) -> Result<(), rusqlite::Error> {
        let (project_id, old_pos): (i64, i32) = self.conn.query_row(
            "SELECT project_id, position FROM columns WHERE id = ?1",
            params![column_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if old_pos == new_position {
            return Ok(());
        }

        if new_position > old_pos {
            self.conn.execute(
                "UPDATE columns SET position = position - 1 WHERE project_id = ?1 AND position > ?2 AND position <= ?3",
                params![project_id, old_pos, new_position],
            )?;
        } else {
            self.conn.execute(
                "UPDATE columns SET position = position + 1 WHERE project_id = ?1 AND position >= ?2 AND position < ?3",
                params![project_id, new_position, old_pos],
            )?;
        }

        self.conn.execute(
            "UPDATE columns SET position = ?1 WHERE id = ?2",
            params![new_position, column_id],
        )?;

        Ok(())
    }

    pub fn update_task(
        &self,
        task_id: i64,
        title: &str,
        description: &str,
        documents: &str,
        link: &str,
        tags: &str,
        due_date: &str,
        priority: i32,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, documents = ?3, link = ?4, tags = ?5, due_date = ?6, priority = ?7 WHERE id = ?8",
            params![title, description, documents, link, tags, due_date, priority, task_id],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, task_id: i64) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        Ok(())
    }

    pub fn get_task(&self, task_id: i64) -> Result<Task, rusqlite::Error> {
        self.conn.query_row(
            "SELECT id, project_id, title, description, documents, link, tags, column_id, position, created_at, due_date, priority FROM tasks WHERE id = ?1",
            params![task_id],
            |row| Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                documents: row.get(4)?,
                link: row.get(5)?,
                tags: row.get(6)?,
                column_id: row.get(7)?,
                position: row.get(8)?,
                created_at: row.get(9)?,
                due_date: row.get(10)?,
                priority: row.get(11)?,
            }),
        )
    }
}
