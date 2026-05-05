use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Value};

use crate::model::{Project, Session, SessionInfo};

pub struct OpencodeBackend {
    pub db_path: PathBuf,
}

impl OpencodeBackend {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    fn open(&self) -> Option<Connection> {
        Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .ok()
    }

    pub fn list_projects(&self) -> Vec<Project> {
        let Some(conn) = self.open() else { return Vec::new() };

        let sql = "SELECT id, worktree, name FROM project \
                   WHERE EXISTS (SELECT 1 FROM session \
                                 WHERE project_id = project.id AND time_archived IS NULL) \
                   ORDER BY time_updated DESC";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let worktree: String = row.get(1)?;
            let name: Option<String> = row.get(2)?;
            Ok((id, worktree, name))
        });

        let mut projects = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                let (id, worktree, name_opt) = r;
                let derived = std::path::Path::new(&worktree)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let name = derived
                    .or(name_opt)
                    .unwrap_or_else(|| id.clone());
                projects.push(Project {
                    id,
                    name,
                    cwd: Some(worktree),
                });
            }
        }
        projects
    }

    pub fn list_sessions(&self, project_id: &str) -> Vec<Session> {
        let Some(conn) = self.open() else { return Vec::new() };
        let sql = "SELECT id, title, time_created FROM session \
                   WHERE project_id = ?1 AND time_archived IS NULL \
                   ORDER BY time_created DESC";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([project_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let time_created: i64 = row.get(2)?;
            Ok((id, title, time_created))
        });

        let mut sessions = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                let (id, title, ts) = r;
                sessions.push(Session {
                    id,
                    project_id: project_id.to_string(),
                    title,
                    timestamp: Some(epoch_ms_to_iso(ts)),
                });
            }
        }
        sessions
    }

    pub fn recent_sessions(&self) -> Vec<Session> {
        let Some(conn) = self.open() else { return Vec::new() };
        let sql = "SELECT id, project_id, title, time_created FROM session \
                   WHERE time_archived IS NULL \
                   ORDER BY time_created DESC LIMIT 10";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let project_id: String = row.get(1)?;
            let title: String = row.get(2)?;
            let time_created: i64 = row.get(3)?;
            Ok((id, project_id, title, time_created))
        });

        let mut sessions = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                let (id, project_id, title, ts) = r;
                sessions.push(Session {
                    id,
                    project_id,
                    title,
                    timestamp: Some(epoch_ms_to_iso(ts)),
                });
            }
        }
        sessions
    }

    pub fn find_session(&self, session_id: &str) -> Option<SessionInfo> {
        let conn = self.open()?;
        let sid = strip_jsonl(session_id);
        let mut stmt = conn
            .prepare("SELECT project_id FROM session WHERE id = ?1")
            .ok()?;
        let project_id: Option<String> = stmt
            .query_row([sid.as_str()], |row| row.get::<_, String>(0))
            .ok();
        project_id.map(|project_id| SessionInfo { project_id })
    }

    pub fn read_session(&self, _project_id: &str, session_id: &str) -> Option<String> {
        let conn = self.open()?;
        let sid = strip_jsonl(session_id);

        // Confirm the session lives in this DB before synthesizing — otherwise
        // the merged Backend::Opencode would short-circuit on the first DB.
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM session WHERE id = ?1 LIMIT 1",
                [sid.as_str()],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return None;
        }

        let messages = load_messages(&conn, &sid).ok()?;
        let mut lines: Vec<String> = Vec::new();

        for (msg_id, msg_time_created, msg_data) in messages {
            let role = msg_data
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("user")
                .to_string();
            let msg_ts_ms = msg_data
                .get("time")
                .and_then(|t| t.get("created"))
                .and_then(|v| v.as_i64())
                .unwrap_or(msg_time_created);

            let parts = load_parts(&conn, &msg_id).ok().unwrap_or_default();

            let mut content_blocks: Vec<Value> = Vec::new();
            let mut buffered_tool_results: Vec<(Value, i64)> = Vec::new();

            for part in parts {
                let ptype = part.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match ptype {
                    "text" => {
                        let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        if text.is_empty() {
                            continue;
                        }
                        content_blocks.push(json!({"type": "text", "text": text}));
                    }
                    "reasoning" => {
                        let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        let signature = part
                            .pointer("/metadata/anthropic/signature")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        content_blocks.push(json!({
                            "type": "thinking",
                            "thinking": text,
                            "signature": signature,
                        }));
                    }
                    "tool" => {
                        let call_id = part
                            .get("callID")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let tool_name = part
                            .get("tool")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let state = part.get("state").cloned().unwrap_or(Value::Null);
                        let input = state.get("input").cloned().unwrap_or(json!({}));
                        let status = state.get("status").and_then(|v| v.as_str()).unwrap_or("");

                        content_blocks.push(json!({
                            "type": "tool_use",
                            "id": call_id,
                            "name": tool_name,
                            "input": input,
                        }));

                        if status == "completed" || status == "error" {
                            let output = state.get("output").cloned().unwrap_or(Value::Null);
                            let content_value = match &output {
                                Value::String(s) => Value::String(s.clone()),
                                Value::Null => Value::String(String::new()),
                                other => Value::String(other.to_string()),
                            };
                            let result_ts = state
                                .pointer("/time/end")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(msg_ts_ms);
                            buffered_tool_results.push((
                                json!({
                                    "type": "tool_result",
                                    "tool_use_id": call_id,
                                    "content": content_value,
                                    "is_error": status == "error",
                                }),
                                result_ts,
                            ));
                        }
                    }
                    "file" => {
                        let mime = part.get("mime").and_then(|v| v.as_str()).unwrap_or("?");
                        let filename = part.get("filename").and_then(|v| v.as_str()).unwrap_or("?");
                        let url = part.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        content_blocks.push(json!({
                            "type": "text",
                            "text": format!("[file: {} ({}) — {}]", filename, mime, url),
                        }));
                    }
                    _ => {}
                }
            }

            if !content_blocks.is_empty() {
                let entry = json!({
                    "type": role,
                    "message": {
                        "id": msg_id,
                        "role": role,
                        "content": content_blocks,
                    },
                    "timestamp": epoch_ms_to_iso(msg_ts_ms),
                    "uuid": msg_id,
                });
                lines.push(entry.to_string());
            }

            for (i, (block, ts)) in buffered_tool_results.into_iter().enumerate() {
                let synthetic_id = format!("{}-tr-{}", msg_id, i);
                let entry = json!({
                    "type": "user",
                    "message": {
                        "id": synthetic_id,
                        "role": "user",
                        "content": [block],
                    },
                    "timestamp": epoch_ms_to_iso(ts),
                    "uuid": synthetic_id,
                });
                lines.push(entry.to_string());
            }
        }

        Some(lines.join("\n"))
    }
}

fn strip_jsonl(s: &str) -> String {
    s.strip_suffix(".jsonl").unwrap_or(s).to_string()
}

fn epoch_ms_to_iso(ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| String::from("1970-01-01T00:00:00Z"))
}

fn load_messages(
    conn: &Connection,
    session_id: &str,
) -> rusqlite::Result<Vec<(String, i64, Value)>> {
    let mut stmt = conn.prepare(
        "SELECT id, time_created, data FROM message \
         WHERE session_id = ?1 ORDER BY time_created, id",
    )?;
    let rows = stmt.query_map([session_id], |row| {
        let id: String = row.get(0)?;
        let time_created: i64 = row.get(1)?;
        let data: String = row.get(2)?;
        let value: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
        Ok((id, time_created, value))
    })?;
    Ok(rows.flatten().collect())
}

fn load_parts(conn: &Connection, message_id: &str) -> rusqlite::Result<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT data FROM part \
         WHERE message_id = ?1 ORDER BY time_created, id",
    )?;
    let rows = stmt.query_map([message_id], |row| {
        let data: String = row.get(0)?;
        Ok(serde_json::from_str::<Value>(&data).unwrap_or(Value::Null))
    })?;
    Ok(rows.flatten().filter(|v| !v.is_null()).collect())
}
