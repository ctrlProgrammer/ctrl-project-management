# Development Guidelines

## Principles

### KISS (Keep It Simple, Stupid)
- Solve the problem at hand, not imagined future ones
- Favor flat structures over deep nesting
- One function = one responsibility
- If it's hard to name, it's doing too much

### Object-Oriented Programming
- Encapsulate behavior with data — don't let internal state leak
- Prefer composition over inheritance
- Program to interfaces, not implementations
- Depend on abstractions; inject concrete dependencies

### Senior Developer Mindset
- Write code for the next reader, not the compiler
- Fewer lines ≠ better code. Clarity wins.
- Fail fast — validate inputs at boundaries
- Test the public contract, not internal details
- Prefer immutable data; when mutation is needed, keep it local and obvious
- Name things by what they do, not how they do it
- Delete unused code on sight

## Conventions
- Keep files small (< 200 lines typical)
- Avoid comments — let the code speak
- No abbreviations in names unless universally understood (ID, URL, HTTP)

## MCP Server

The same binary also serves as an MCP server. Run with the `mcp` subcommand:

```
ctrl-project-management mcp
```

### Configuration
The project's `opencode.json` registers the MCP server at the release binary path.

### Tools
- `list_projects` — list all projects
- `get_columns` — get columns for a project (requires `project_id`)
- `list_tasks` — list tasks with optional `year`/`month` filter (requires `project_id`)
- `get_task` — get a single task by `task_id`
- `create_task` — create a task (requires `project_id`, `column_id`, `title`)
- `update_task` — update task fields (requires `task_id`)
- `create_project` — create a new project (requires `name`)
- `create_column` — add a column to a project (requires `project_id`, `name`)

No delete tools are exposed.

### Build release artifacts
```
./scripts/release.sh [version]
```

Produces `.tar.gz` and `.deb` in the project root.
