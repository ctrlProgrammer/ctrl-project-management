pub const APP_CSS: &str = r#"
    window {
        background-color: transparent;
    }
    .widget-container {
        background-color: rgba(28, 28, 34, 0.94);
        border-radius: 20px;
        margin: 10px;
    }
    .header-box {
        border-bottom: 0.5px solid #3A3A3C;
        padding: 8px 14px;
        margin: 0 6px;
    }
    .drag-handle {
        min-height: 24px;
        border-bottom: 0.5px solid rgba(58, 58, 60, 0.4);
    }
    .project-label {
        font-size: 11px;
        font-weight: 600;
        color: #8E8E93;
        margin-right: 4px;
        letter-spacing: 0.5px;
    }
    .project-selector {
        background-color: #3A3A3C;
        color: #EDEDED;
        border-radius: 8px;
        min-height: 28px;
        min-width: 160px;
        padding: 0 12px;
        border: none;
        outline: none;
        box-shadow: none;
        text-shadow: none;
        font-size: 13px;
        font-weight: 500;
    }
    .project-selector:hover {
        background-color: #48484A;
        box-shadow: none;
    }
    .project-popover {
        background-color: #252529;
        border-radius: 14px;
        padding: 6px;
        box-shadow: none;
    }
    .project-selector button {
        box-shadow: none;
        outline: none;
        border: none;
    }

    .project-search {
        background-color: #3A3A3C;
        color: #EDEDED;
        border-radius: 8px;
        padding: 6px 10px;
        min-height: 28px;
        border: none;
        outline: none;
        box-shadow: none;
        margin: 4px;
    }
    .project-search:focus {
        background-color: #404042;
        outline: none;
    }
    .project-list {
        background-color: transparent;
        border: none;
        margin: 0;
        padding: 0;
    }
    .project-list row {
        background-color: transparent;
        border-radius: 6px;
        padding: 0 4px;
        min-height: 32px;
        border: none;
        outline: none;
    }
    .project-list row:hover {
        background-color: rgba(255, 255, 255, 0.06);
    }
    .project-list row:focus,
    .project-list row:selected {
        background-color: #FF9F0A;
        color: #1C1C1E;
        border-radius: 6px;
        outline: none;
    }
    .project-list label {
        padding: 4px 8px;
        font-size: 13px;
        font-weight: 500;
    }
    .kanban-column {
        background-color: #252529;
        border: 0.5px solid #3A3A3C;
        border-radius: 16px;
        margin: 6px;
    }
    .kanban-column:drop(active) {
        background-color: rgba(255, 159, 10, 0.08);
    }
    .kanban-column.column-accent-0:drop(active) { border-color: #FF9F0A; }
    .kanban-column.column-accent-1:drop(active) { border-color: #32ADE6; }
    .kanban-column.column-accent-2:drop(active) { border-color: #AF52DE; }
    .kanban-column.column-accent-3:drop(active) { border-color: #34C759; }
    .column-header-box {
        padding: 14px 14px 0;
    }
    .column-header {
        font-size: 13px;
        font-weight: 700;
        color: #EDEDED;
        letter-spacing: 0.3px;
    }
    .column-count {
        font-size: 11px;
        font-weight: 500;
        color: #8E8E93;
        padding: 0 14px 6px;
        letter-spacing: 0.3px;
    }
    .column-delete {
        background: none;
        color: #EDEDED;
        font-weight: 600;
        font-size: 11px;
        min-width: 22px;
        min-height: 22px;
        padding: 0;
        border: none;
        border-radius: 6px;
    }
    .column-delete:hover {
        color: #CC2F26;
        background-color: rgba(204, 47, 38, 0.15);
    }
    .delete-project-btn {
        padding: 0 10px;
    }
    .task-card {
        background-color: rgba(37, 37, 41, 0.9);
        border-radius: 12px;
        margin: 4px 10px;
        padding: 10px 12px;
        border: 0.5px solid #3A3A3C;
    }
    .column-accent-0 .task-card:hover { border-color: #FF9F0A; }
    .column-accent-1 .task-card:hover { border-color: #32ADE6; }
    .column-accent-2 .task-card:hover { border-color: #AF52DE; }
    .column-accent-3 .task-card:hover { border-color: #34C759; }
    .task-card:hover {
        background-color: rgba(58, 58, 60, 0.9);
    }
    .task-title {
        font-size: 13px;
        font-weight: 500;
        color: #EDEDED;
    }
    .task-delete {
        background: none;
        color: #EDEDED;
        font-weight: 600;
        font-size: 12px;
        min-width: 24px;
        min-height: 24px;
        padding: 0;
        border: none;
        border-radius: 6px;
    }
    .task-delete:hover {
        color: #CC2F26;
        background-color: rgba(204, 47, 38, 0.15);
    }
    .task-edit {
        background: none;
        color: #8E8E93;
        font-size: 12px;
        min-width: 24px;
        min-height: 24px;
        padding: 0;
        border: none;
        border-radius: 6px;
    }
    .task-edit:hover {
        color: #32ADE6;
        background-color: rgba(50, 173, 230, 0.15);
    }
    .task-desc {
        font-size: 11px;
        font-weight: 400;
        color: #8E8E93;
    }
    .task-link {
        font-size: 11px;
        font-weight: 400;
        color: #32ADE6;
    }
    .task-link-icon {
        font-size: 11px;
    }
    .task-docs {
        font-size: 11px;
        font-weight: 400;
        color: #8E8E93;
    }
    .task-tag {
        font-size: 10px;
        font-weight: 600;
        color: #1C1C1E;
        background-color: #FF9F0A;
        border-radius: 4px;
        padding: 1px 6px;
    }
    .task-tag:nth-child(3n+1) { background-color: #FF9F0A; }
    .task-tag:nth-child(3n+2) { background-color: #32ADE6; }
    .task-tag:nth-child(3n+3) { background-color: #AF52DE; }
    .dialog-docs-label {
        font-size: 11px;
        font-weight: 400;
        color: #8E8E93;
    }
    .dialog-drop-area {
        border: 1.5px dashed #3A3A3C;
        border-radius: 8px;
        min-height: 60px;
        padding: 8px;
        background-color: rgba(58, 58, 60, 0.15);
    }
    .dialog-drop-area:drop(active) {
        border-color: #FF9F0A;
        background-color: rgba(255, 159, 10, 0.08);
    }
    .dialog-drop-area label {
        font-size: 12px;
        color: #8E8E93;
    }
    .column-accent-0 .add-task-button { color: #FF9F0A; }
    .column-accent-1 .add-task-button { color: #32ADE6; }
    .column-accent-2 .add-task-button { color: #AF52DE; }
    .column-accent-3 .add-task-button { color: #34C759; }
    .add-column-button, .add-task-button {
        font-size: 13px;
        font-weight: 500;
        padding: 8px;
        border-radius: 16px;
        min-height: 28px;
    }
    .add-column-button:hover, .add-task-button:hover {
        background-color: rgba(37, 37, 41, 0.6);
    }
    .add-column-outer {
        color: #FF9F0A;
        font-size: 13px;
        font-weight: 500;
        padding: 12px 16px;
        border-radius: 16px;
        margin: 6px;
        border: 0.5px dashed #3A3A3C;
        min-width: 180px;
        background-color: transparent;
    }
    .add-column-outer:hover {
        background-color: rgba(37, 37, 41, 0.6);
        border-color: #FF9F0A;
    }

    .dialog-window {
        background-color: #1C1C22;
    }
    .dialog-content {
        background-color: #1C1C22;
        border-radius: 12px;
    }
    .dialog-title {
        font-size: 15px;
        font-weight: 700;
        color: #EDEDED;
    }
    .dialog-entry {
        background-color: #3A3A3C;
        color: #EDEDED;
        border-radius: 8px;
        padding: 6px 10px;
        min-height: 28px;
        border: none;
    }
    .dialog-entry:focus {
        background-color: #404042;
    }
    .dialog-textview {
        background-color: transparent;
        color: #EDEDED;
        font-family: inherit;
        font-size: 13px;
        border: none;
    }
    .dialog-textview text {
        background-color: transparent;
    }
    .accent-button {
        background-color: #FF9F0A;
        color: #1C1C1E;
        border-radius: 8px;
        padding: 6px 16px;
        border: none;
        font-weight: 600;
        font-size: 13px;
    }
    .accent-button:hover {
        background-color: #FFB340;
    }

    .filter-box {
        margin: 0 12px;
    }
    .filter-label {
        font-size: 12px;
        font-weight: 600;
        color: #EDEDED;
        min-width: 100px;
        text-align: center;
    }
    .month-nav-btn {
        background: none;
        color: #8E8E93;
        font-size: 16px;
        font-weight: 600;
        min-width: 26px;
        min-height: 22px;
        padding: 0;
        border: none;
        border-radius: 6px;
    }
    .month-nav-btn:hover {
        color: #EDEDED;
        background-color: rgba(255, 255, 255, 0.08);
    }
    .current-month-btn {
        background: none;
        color: #FF9F0A;
        font-size: 11px;
        font-weight: 600;
        min-height: 22px;
        padding: 0 8px;
        border: none;
        border-radius: 6px;
    }
    .current-month-btn:hover {
        color: #EDEDED;
        background-color: rgba(255, 159, 10, 0.2);
    }

"#;
