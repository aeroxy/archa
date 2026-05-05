## Architecture

- **Rust backend** (axum + tokio):
  - Reads two sources via a `Backend` enum: Claude (`~/.claude/projects/*.jsonl`) and OpenCode (`~/.local/share/opencode/opencode*.db`, SQLite).
  - For OpenCode, synthesizes Claude-style JSONL on the fly so the frontend renderer stays source-agnostic.
  - Extracts project names from `cwd` (Claude) or `project.worktree` (OpenCode) for high accuracy.
  - Serves a REST API for project/session discovery and content reading.
  - Embeds the frontend via `rust-embed` for a single-binary experience.
- **React frontend** (Vite + TypeScript + Tailwind):
  - Three-column layout (Switcher, Explorer, Reader).
  - Uses `react-markdown` with `remark-gfm` for high-fidelity rendering.
  - Custom typography system (Inter for UI, Newsreader for Content, Space Grotesk for Code).
  - Renders `ContentBlock[]` messages: `text`, `thinking` (collapsible `ThinkingWidget`), `tool_use`, and `tool_result` blocks.
  - Uses `lucide-react` for SVG icons.

## Key Directories

- `src/main.rs` — Axum wiring and route handlers.
- `src/backend.rs` — `Backend` enum dispatch + multi-DB discovery for OpenCode.
- `src/claude.rs` — fs-based Claude reader.
- `src/opencode.rs` — read-only SQLite reader + Claude-shape JSONL synthesizer.
- `src/model.rs` — shared `Project` / `Session` / `SessionInfo` types.
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

## Preparing a new release

1. Bump the version using `make bump-patch`, `make bump-minor` or `make bump-major`.
2. Build the release binary: `make build`
3. Zip the binary inside the release folder: `zip -j target/release/archa_macos_arm64.zip target/release/archa`
4. Calculate the SHA256: `shasum -a 256 target/release/archa_macos_arm64.zip`
5. Update `Formula/archa.rb` with the new version, URL, and SHA256.

## Wiki

| Doc | Covers |
|-----|--------|
| [wiki/architecture.md](wiki/architecture.md) | How Archa routes between Claude/OpenCode backends, parses sessions, and renders tool calls. |
| [wiki/opencode.md](wiki/opencode.md) | OpenCode SQLite storage layout, JSONL synthesis mapping, multi-DB merging. |
