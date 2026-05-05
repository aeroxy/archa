use std::fs;
use std::path::{Path as StdPath, PathBuf};
use std::time::SystemTime;

use crate::model::{Project, Session, SessionInfo};

pub struct ClaudeBackend {
    pub root: PathBuf,
}

impl ClaudeBackend {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn list_projects(&self) -> Vec<Project> {
        let mut projects = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let Ok(id) = entry.file_name().into_string() else { continue };

                let mut cwd = None;
                let mut has_sessions = false;
                if let Ok(sessions) = fs::read_dir(entry.path()) {
                    for session in sessions.flatten() {
                        if session.path().extension().and_then(|s| s.to_str()) == Some("jsonl") {
                            has_sessions = true;
                            if cwd.is_none() {
                                if let Some(found_cwd) = extract_cwd_from_file(&session.path()) {
                                    cwd = Some(found_cwd);
                                }
                            }
                        }
                    }
                }

                if !has_sessions {
                    continue;
                }

                let name = cwd.as_ref()
                    .and_then(|p| StdPath::new(p).file_name())
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| id.replace("-", "/"));

                projects.push(Project { id, name, cwd });
            }
        }

        projects
    }

    pub fn list_sessions(&self, project_id: &str) -> Vec<Session> {
        let path = self.root.join(project_id);
        let mut sessions = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    let id = entry.file_name().to_string_lossy().into_owned();
                    let title = get_session_title(&file_path).unwrap_or_else(|| id.clone());
                    sessions.push(Session {
                        id,
                        project_id: project_id.to_string(),
                        title,
                        timestamp: None,
                    });
                }
            }
        }

        sessions
    }

    pub fn recent_sessions(&self) -> Vec<Session> {
        let mut all_sessions = Vec::new();

        if let Ok(project_entries) = fs::read_dir(&self.root) {
            for project_entry in project_entries.flatten() {
                if !project_entry.path().is_dir() {
                    continue;
                }
                let project_id = project_entry.file_name().to_string_lossy().into_owned();
                if let Ok(session_entries) = fs::read_dir(project_entry.path()) {
                    for session_entry in session_entries.flatten() {
                        let file_path = session_entry.path();
                        if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                            let mtime = fs::metadata(&file_path)
                                .and_then(|m| m.modified())
                                .unwrap_or(SystemTime::UNIX_EPOCH);
                            all_sessions.push((mtime, project_id.clone(), file_path));
                        }
                    }
                }
            }
        }

        all_sessions.sort_by(|a, b| b.0.cmp(&a.0));

        let mut recent = Vec::new();
        for (_mtime, project_id, file_path) in all_sessions.into_iter().take(10) {
            let id = file_path.file_name().unwrap().to_string_lossy().into_owned();
            let title = get_session_title(&file_path).unwrap_or_else(|| id.clone());
            recent.push(Session {
                id,
                project_id,
                title,
                timestamp: None,
            });
        }

        recent
    }

    pub fn find_session(&self, session_id: &str) -> Option<SessionInfo> {
        if let Ok(project_entries) = fs::read_dir(&self.root) {
            for project_entry in project_entries.flatten() {
                if !project_entry.path().is_dir() {
                    continue;
                }
                let project_id = project_entry.file_name().to_string_lossy().into_owned();
                let direct = project_entry.path().join(session_id);
                let with_ext = project_entry.path().join(format!("{}.jsonl", session_id));
                if direct.exists() || with_ext.exists() {
                    return Some(SessionInfo { project_id });
                }
            }
        }
        None
    }

    pub fn read_session(&self, project_id: &str, session_id: &str) -> Option<String> {
        let path = self.root.join(project_id).join(session_id);
        fs::read_to_string(path).ok()
    }
}

fn extract_cwd_from_file(path: &StdPath) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
                return Some(cwd.to_string());
            }
        }
    }
    None
}

fn get_session_title(path: &StdPath) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(msg) = v.get("message") {
                if let Some(content) = msg.get("content") {
                    let text = if let Some(text) = content.as_str() {
                        text.to_string()
                    } else if let Some(arr) = content.as_array() {
                        arr.iter()
                            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else {
                        continue;
                    };

                    if !text.is_empty() && !text.contains("<local-command-caveat>") {
                        return Some(text.chars().take(80).collect());
                    }
                }
            }
        }
    }
    None
}
