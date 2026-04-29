# Architecture

Archa is designed as a lightweight wrapper around the Claude session filesystem.

## Data Discovery

Claude Code and Claude Desktop store session data in `~/.claude/projects/`. Each project is a directory named with a slugified version of its original path. Inside these directories are `.jsonl` files, where each line represents a message or a system event.

Archa performs the following steps to reconstruct the project list:
1. Iterates over subdirectories in `~/.claude/projects/`.
2. For each directory, it reads the first `.jsonl` file it finds.
3. It extracts the `cwd` field from the JSON metadata.
4. It uses the `basename` of the `cwd` as the project's display name, ensuring that names like `chrome-devtools-cli` are preserved exactly as they are on the filesystem.

## Backend (Rust)

The backend is built with **Axum** and **Tokio**. It serves two primary purposes:
1. **REST API**: Providing endpoints for project listing, session listing, and raw session data reading.
2. **Static Asset Serving**: Using `rust-embed` to serve the React frontend directly from the binary.

### Key Endpoints
- `GET /api/projects`: Returns a list of all detected projects with their IDs and names.
- `GET /api/sessions/{project_id}`: Returns all `.jsonl` session files for a specific project.
- `GET /api/session/{project_id}/{session_id}`: Returns the raw content of a specific session file.
- `GET /api/recent-sessions`: Returns the 10 most recently modified sessions across all projects.

## Frontend (React)

The frontend is a modern React application built with **Vite** and **Tailwind CSS**.

### Three-Column Design
- **Column 1 (Switcher)**: Allows switching between different "Contexts of Interaction" (currently focused on Claude).
- **Column 2 (Explorer)**: A hierarchical view of projects and their sessions.
- **Column 3 (Reader)**: A Markdown-rendered view of the selected conversation.

### Message Data Model

Messages are represented as a `ContentBlock[]` array rather than a flat string. Each block has a `type` field:

| Type | Description |
|------|-------------|
| `text` | Plain assistant or user text. |
| `thinking` | Claude's internal reasoning, rendered in a collapsible `ThinkingWidget`. |
| `tool_use` | A tool invocation (name + JSON input), styled with a primary/blue accent. |
| `tool_result` | The tool's response, remapped back into the originating assistant message and styled green (success) or red (error). |

`convertToMessages` in `App.tsx` parses raw `.jsonl` lines and produces this structure.

### Streaming Chunk Merging

Claude's streaming logs emit multiple partial JSON lines for a single logical message. Archa merges consecutive chunks by matching `message.id` (assistant turns) or `prompt_id` (user turns) so that each chat bubble is complete and non-fragmented. Empty messages (whitespace-only after merging) are suppressed entirely.

### Tool Result Remapping

`tool_result` blocks arrive in the `.jsonl` stream as separate `user`-role events. Archa's parser maps each result back to the assistant message that issued the matching `tool_use` call (by `tool_use_id`), keeping the call/result pair visually together in the Reader column.

### Markdown Rendering
We use `react-markdown` with `remark-gfm` to handle technical content, tables, and task lists. The typography is carefully chosen to distinguish between UI elements (Sans-serif) and the reading experience (Serif).

### Markdown Export

`exportMarkdown` serializes the `ContentBlock[]` structure into clean Markdown. Speaker labels use "User" / "Assistant". Tool calls and results are formatted as fenced code blocks. Excessive blank lines are trimmed.

## Backend Notes

### Route Parameter Syntax
Axum v0.8+ requires `{param}` syntax (not `:param`). The route for CLI asset passthrough was corrected from `/:cli` to `/{cli}` to prevent a startup panic.

### CLI Flag Conflict
The `--port` flag uses `-p`. The `--projects_path` flag was changed to `-d` to avoid the conflict.

### Empty-Folder Filtering
`list_projects` skips project directories that contain no `.jsonl` files. This prevents ghost entries from appearing in the Explorer (e.g., `.timelines` system folders or empty clones).
