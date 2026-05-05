# OpenCode Support

Archa reads [OpenCode](https://github.com/anomalyco/opencode) sessions in addition to Claude Code's. Unlike Claude — which writes one `.jsonl` per session under `~/.claude/projects/` — OpenCode stores everything in **SQLite databases** under `~/.local/share/opencode/`.

## Storage layout

OpenCode keeps its data in a small set of SQLite files:

```
~/.local/share/opencode/
├── opencode.db        # production binary
├── opencode-dev.db    # `opencode-dev` build
├── opencode-local.db  # `opencode-local` build
└── storage/           # legacy JSON files (no longer canonical)
```

Archa discovers all `opencode*.db` files at startup and merges them under a single `opencode` backend. Lookups fan out across each DB until one returns a hit.

### Schema (relevant tables)

```sql
project (id, worktree, name, time_created, time_updated)
session (id, project_id, slug, directory, title, time_created, time_archived)
message (id, session_id, time_created, data TEXT)   -- data: JSON
part    (id, message_id, session_id, time_created, data TEXT)  -- data: JSON
```

`message.data` holds `{role, time:{created}, agent, model, ...}`. `part.data` is tagged by `type`: `text`, `reasoning`, `tool`, `file`, `step-start`, `step-finish`.

All queries open the DB read-only with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI`, so Archa can read while OpenCode is running (WAL mode handles concurrent readers).

## JSONL synthesis

The frontend's renderer expects Claude-style JSONL — `{type:'user'|'assistant', message:{content: ContentBlock[]}, timestamp, uuid}`. Rather than build a parallel renderer, the OpenCode backend synthesizes Claude-shape JSONL on the fly. Mapping:

| OpenCode `part.type` | Synthesized block | Notes |
|---|---|---|
| `text` | `{type:'text', text}` | empty text dropped |
| `reasoning` | `{type:'thinking', thinking, signature}` | signature pulled from `metadata.anthropic.signature` |
| `tool` | `{type:'tool_use', id: callID, name: tool, input: state.input}` plus a synthetic user line carrying `{type:'tool_result', tool_use_id: callID, content, is_error}` | only emitted if `state.status` is `completed` or `error` — pending tools emit `tool_use` only |
| `file` | `{type:'text', text:"[file: <name> (<mime>) — <url>]"}` | |
| `step-start` / `step-finish` | dropped | UI markers, no content |

The split tool_use / tool_result pattern matches how Claude's JSONL encodes tool calls, so the frontend's existing `toolResultsMap` re-attachment ([App.tsx:81-110](../frontend/src/App.tsx#L81-L110)) works without modification.

## Multi-DB merging

`Backend::Opencode` holds a `Vec<OpencodeBackend>` — one per discovered DB. Per-method behavior:

- **list_projects**: dedupe by project id; first DB wins (primary `opencode.db` is sorted first).
- **list_sessions / recent_sessions**: union sessions across all DBs, dedupe by session id, sort by `time_created DESC`.
- **find_session / read_session**: try each DB in order; return on first hit.

For `read_session` to short-circuit correctly, each `OpencodeBackend::read_session` must return `None` (not `Some("")`) when the session isn't in that DB. The implementation does an explicit `SELECT 1 FROM session WHERE id=?` existence check before synthesizing.

## Routes

OpenCode reuses the same `{cli}` API surface as Claude:

```
GET /api/_/backends                           → ["claude", "opencode"]
GET /api/opencode/projects                    → merged across all DBs
GET /api/opencode/sessions/{project_id}       → merged
GET /api/opencode/session/{pid}/{sid}         → synthesized JSONL
GET /api/opencode/recent-sessions             → merged, top 10
GET /api/opencode/session-info/{sid}          → {project_id}
```

Deep links use the same shape as Claude — `http://localhost:<port>/opencode/<session_id>` — so a session id copied from OpenCode itself can be pasted directly into Archa's URL.

## Module layout

OpenCode support is isolated in [src/opencode.rs](../src/opencode.rs); the `Backend` enum in [src/backend.rs](../src/backend.rs) routes between Claude and OpenCode and handles multi-DB fan-out. [src/claude.rs](../src/claude.rs) is unchanged from the original single-source implementation.
