use std::path::PathBuf;
use std::sync::OnceLock;

use std::collections::HashMap;

use crate::claude::ClaudeBackend;
use crate::model::{Project, Session, SessionInfo};
use crate::opencode::OpencodeBackend;

pub enum Backend {
    Claude(ClaudeBackend),
    Opencode(Vec<OpencodeBackend>),
}

impl Backend {
    pub fn from_cli(cli: &str, state: &AppState) -> Option<Self> {
        if cli == "claude" {
            let path = state
                .claude_root
                .clone()
                .or_else(|| home::home_dir().map(|h| h.join(".claude/projects")))?;
            return Some(Backend::Claude(ClaudeBackend::new(path)));
        }

        if cli == "opencode" {
            let dbs: Vec<OpencodeBackend> = state
                .opencode_dbs()
                .iter()
                .map(|(_, p)| OpencodeBackend::new(p.clone()))
                .collect();
            if dbs.is_empty() {
                return None;
            }
            return Some(Backend::Opencode(dbs));
        }

        None
    }

    pub fn list_projects(&self) -> Vec<Project> {
        match self {
            Backend::Claude(b) => b.list_projects(),
            Backend::Opencode(dbs) => {
                // Dedupe by project id; first DB wins (primary opencode.db is first).
                let mut seen: HashMap<String, Project> = HashMap::new();
                let mut order: Vec<String> = Vec::new();
                for db in dbs {
                    for p in db.list_projects() {
                        if !seen.contains_key(&p.id) {
                            order.push(p.id.clone());
                            seen.insert(p.id.clone(), p);
                        }
                    }
                }
                order.into_iter().filter_map(|id| seen.remove(&id)).collect()
            }
        }
    }

    pub fn list_sessions(&self, project_id: &str) -> Vec<Session> {
        match self {
            Backend::Claude(b) => b.list_sessions(project_id),
            Backend::Opencode(dbs) => {
                let mut all: Vec<Session> = Vec::new();
                let mut seen_ids = std::collections::HashSet::new();
                for db in dbs {
                    for s in db.list_sessions(project_id) {
                        if seen_ids.insert(s.id.clone()) {
                            all.push(s);
                        }
                    }
                }
                all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                all
            }
        }
    }

    pub fn recent_sessions(&self) -> Vec<Session> {
        match self {
            Backend::Claude(b) => b.recent_sessions(),
            Backend::Opencode(dbs) => {
                let mut all: Vec<Session> = Vec::new();
                let mut seen_ids = std::collections::HashSet::new();
                for db in dbs {
                    for s in db.recent_sessions() {
                        if seen_ids.insert(s.id.clone()) {
                            all.push(s);
                        }
                    }
                }
                all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                all.truncate(10);
                all
            }
        }
    }

    pub fn find_session(&self, session_id: &str) -> Option<SessionInfo> {
        match self {
            Backend::Claude(b) => b.find_session(session_id),
            Backend::Opencode(dbs) => dbs.iter().find_map(|db| db.find_session(session_id)),
        }
    }

    pub fn read_session(&self, project_id: &str, session_id: &str) -> Option<String> {
        match self {
            Backend::Claude(b) => b.read_session(project_id, session_id),
            Backend::Opencode(dbs) => dbs
                .iter()
                .find_map(|db| db.read_session(project_id, session_id)),
        }
    }
}

pub struct AppState {
    pub claude_root: Option<PathBuf>,
    pub opencode_root: PathBuf,
    opencode_db_cache: OnceLock<Vec<(String, PathBuf)>>,
}

impl AppState {
    pub fn new(claude_root: Option<PathBuf>) -> Self {
        let opencode_root = home::home_dir()
            .map(|h| h.join(".local/share/opencode"))
            .unwrap_or_else(|| PathBuf::from("/tmp/opencode-missing"));
        Self {
            claude_root,
            opencode_root,
            opencode_db_cache: OnceLock::new(),
        }
    }

    pub fn opencode_dbs(&self) -> &Vec<(String, PathBuf)> {
        self.opencode_db_cache.get_or_init(|| discover_opencode_dbs(&self.opencode_root))
    }

    pub fn backend_ids(&self) -> Vec<String> {
        let mut ids = vec!["claude".to_string()];
        if !self.opencode_dbs().is_empty() {
            ids.push("opencode".to_string());
        }
        ids
    }
}

fn discover_opencode_dbs(root: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else { return found };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else { continue };
        if !name.ends_with(".db") {
            continue;
        }
        let stem = match name.strip_suffix(".db") {
            Some(s) => s,
            None => continue,
        };
        // Only opencode*.db, no sidecar files (-shm, -wal already excluded by ends_with(".db"))
        if !stem.starts_with("opencode") {
            continue;
        }
        // Map "opencode" → cli "opencode"; "opencode-dev" → "opencode-dev"; etc.
        let cli_id = stem.to_string();
        found.push((cli_id, path));
    }

    // Stable order, with primary "opencode" first.
    found.sort_by(|a, b| {
        let ord = |id: &str| if id == "opencode" { 0 } else { 1 };
        ord(&a.0).cmp(&ord(&b.0)).then_with(|| a.0.cmp(&b.0))
    });
    found
}
