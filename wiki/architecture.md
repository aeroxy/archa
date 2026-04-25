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

### Markdown Rendering
We use `react-markdown` with `remark-gfm` to handle technical content, tables, and task lists. The typography is carefully chosen to distinguish between UI elements (Sans-serif) and the reading experience (Serif).
