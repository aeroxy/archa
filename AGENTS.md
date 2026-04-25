# Archa

Archa is a lightweight, local-first agent session reader and explorer. It allows you to browse your agent conversation history through a beautiful, three-column academic interface and export sessions to Markdown.

## Project Overview

Archa aims to turn AI interactions into a searchable, readable, and preservable form of technical literature. It runs as a local web server that reads directly from Claude's internal session storage (`~/.claude/projects`).

## Architecture

- **Rust backend** (axum + tokio): 
  - Reads `~/.claude/projects` directly.
  - Extracts project names from `cwd` metadata in `.jsonl` files for high accuracy.
  - Serves a REST API for project/session discovery and content reading.
  - Embeds the frontend via `rust-embed` for a single-binary experience.
- **React frontend** (Vite + TypeScript + Tailwind):
  - Three-column layout (Switcher, Explorer, Reader).
  - Uses `react-markdown` with `remark-gfm` for high-fidelity rendering.
  - Custom typography system (Inter for UI, Newsreader for Content, Space Grotesk for Code).

## Key Directories

- `src/main.rs` — Unified entry point. Handles API routes, static asset serving, and dynamic port selection.
- `frontend/` — React application source.
  - `src/App.tsx` — Main application logic and three-column layout.
  - `tailwind.config.js` — Color and typography definitions matching `DESIGN.md`.
- `wiki/` — Detailed documentation.

## Build & Run

Archa includes a `Makefile` for streamlined operations:

```bash
# Start backend and frontend in parallel for development (with HMR)
make dev

# Build the production binary (frontend dist + release binary)
make build

# Run the production binary
make run
```

## Conventions

- **Package Manager**: Use `bun` (not npm or yarn) for all frontend-related tasks. This is enforced in `frontend/package.json`.
- **Project Naming**: Projects are identified by their true `cwd` rather than just the slugified folder name.
- **Style**: Adheres to the "Archa" design system (Academic Precision / Industrial Utility).
- **Static Assets**: All frontend assets are embedded in the Rust binary during release builds.

## Wiki

| Doc | Covers |
|-----|--------|
| [wiki/architecture.md](wiki/architecture.md) | Deep dive into how Archa reads Claude data and manages state. |
| [wiki/design.md](wiki/design.md) | The philosophy behind the UI and typography choices. |
