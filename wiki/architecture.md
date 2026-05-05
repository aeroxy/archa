# Architecture

Archa is a lightweight wrapper around local agent session storage. It reads two sources today: **Claude Code** (`.jsonl` files on disk) and **OpenCode** (SQLite databases). The frontend renders both through a single `ContentBlock` model.

## Data sources

| CLI | Location | Format |
|---|---|---|
| `claude` | `~/.claude/projects/{project_id}/{session_id}.jsonl` | One JSONL line per message |
| `opencode` | `~/.local/share/opencode/opencode*.db` | SQLite (`project`, `session`, `message`, `part` tables) |

Each source is a `Backend` variant ([src/backend.rs](../src/backend.rs)); the routing fork from `cli` path segment to backend lives in `Backend::from_cli`. Adding a third source is a new enum variant + module — handlers and frontend code don't change.

### Claude project discovery

1. Iterate subdirectories of `~/.claude/projects/`.
2. For each, read the first `.jsonl` and extract its `cwd` field.
3. Use `basename(cwd)` as the project's display name (so e.g. `chrome-devtools-cli` is preserved instead of the slugified directory name).
4. Skip directories with no `.jsonl` files (avoids ghost entries from `.timelines` system folders or empty clones).

### OpenCode project discovery

1. Glob `~/.local/share/opencode/opencode*.db` at startup; cache the list in `AppState` via `OnceLock`.
2. For each DB, query `project` joined with `session` to find non-archived projects.
3. Merge across DBs; dedupe by project id (first DB wins).

See [opencode.md](opencode.md) for the SQLite schema, JSONL synthesis mapping, and multi-DB merging details.

## Backend (Rust)

Built with **Axum** + **Tokio**. Modules:

- [src/main.rs](../src/main.rs) — axum wiring and route handlers.
- [src/backend.rs](../src/backend.rs) — `Backend` enum, `AppState`, multi-DB discovery.
- [src/model.rs](../src/model.rs) — shared `Project` / `Session` / `SessionInfo` types.
- [src/claude.rs](../src/claude.rs) — fs-based Claude reader.
- [src/opencode.rs](../src/opencode.rs) — read-only SQLite reader and Claude-shape JSONL synthesizer.

`rusqlite` is bundled, so the binary has no system SQLite dependency.

### REST API

```
GET /api/_/backends                                → ["claude", "opencode"]
GET /api/{cli}/projects                            → Vec<Project>
GET /api/{cli}/sessions/{project_id}               → Vec<Session>
GET /api/{cli}/session/{project_id}/{session_id}   → raw JSONL (text)
GET /api/{cli}/recent-sessions                     → Vec<Session>
GET /api/{cli}/session-info/{session_id}           → {project_id}
```

For OpenCode, `read_session` returns synthesized Claude-style JSONL — the frontend renderer is source-agnostic.

Notes:
- Axum v0.8+ requires `{param}` syntax (not `:param`).
- The `--port` flag uses `-p`; `--projects-path` uses `-d` to avoid the conflict.

## Frontend (React)

Built with **Vite** + **Tailwind CSS**. The bundle is embedded in the Rust binary via `rust-embed`, so the production deploy is a single executable.

### Three-column layout

- **Column 1 (Switcher)**: One pill per backend, dynamically populated from `/api/_/backends`.
- **Column 2 (Explorer)**: Hierarchical view of projects and their sessions.
- **Column 3 (Reader)**: Markdown-rendered conversation.

### Message data model

Messages are `ContentBlock[]` rather than flat strings. Each block has a `type`:

| Type | Description |
|------|-------------|
| `text` | Plain assistant or user text. |
| `thinking` | Internal reasoning, rendered in a collapsible `ThinkingWidget`. |
| `tool_use` | A tool invocation (name + JSON input). |
| `tool_result` | The tool's response, remapped back into the originating assistant message. |

`convertToMessages` ([App.tsx:45-152](../frontend/src/App.tsx#L45-L152)) parses Claude-style JSONL into this structure. It handles two non-trivial cases:

- **Streaming chunk merging**: streaming logs emit multiple partial JSON lines for one logical message; consecutive chunks with the same `message.id` (assistant) or `prompt_id` (user) are merged.
- **Tool result remapping**: `tool_result` blocks arrive as separate `user`-role events. A `toolResultsMap` keyed by `tool_use_id` re-attaches each result to its originating assistant message so call/result pairs render together.

### Markdown rendering

`react-markdown` + `remark-gfm` for content. Custom component overrides for `<pre>`, `<code>`, and `<table>` keep wide content scrolling internally instead of pushing the reader column wide.

### Markdown export

`exportMarkdown` serializes `ContentBlock[]` into clean Markdown — speaker labels are "User" / "Claude" / "Opencode" depending on the active backend, tool calls and results render as fenced code blocks.
