use axum::{routing::get, Router, extract::Path, response::{IntoResponse, Response}, http::{header, StatusCode, Uri}, body::Body};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use rust_embed::Embed;
use serde::Serialize;
use std::fs;
use std::path::{Path as StdPath, PathBuf};
use tokio::net::TcpListener;
use std::time::SystemTime;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Custom path to Claude projects
    #[arg(short, long)]
    projects_path: Option<PathBuf>,
}

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Assets;

#[derive(Serialize, Clone)]
struct Project {
    name: String,
    id: String,
    cwd: Option<String>,
}

#[derive(Serialize)]
struct Session {
    id: String,
    project_id: String,
    title: String,
    timestamp: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/projects", get(list_projects))
        .route("/api/sessions/{project_id}", get(list_sessions))
        .route("/api/session/{project_id}/{session_id}", get(read_session))
        .route("/api/recent-sessions", get(recent_sessions))
        .fallback(static_handler)
        .layer(CorsLayer::permissive());

    let mut port = args.port;
    let listener = loop {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr).await {
            Ok(listener) => break listener,
            Err(_) => {
                port += 1;
                if port > args.port + 1000 {
                    panic!("Could not find an available port");
                }
            }
        }
    };

    let addr = listener.local_addr().unwrap();
    println!("Archa is running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

fn get_projects_path() -> PathBuf {
    home::home_dir().unwrap().join(".claude/projects")
}

async fn list_projects() -> axum::Json<Vec<Project>> {
    let path = get_projects_path();
    let mut projects = Vec::new();
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(id) = entry.file_name().into_string() {
                    let mut cwd = None;
                    // Try to find the actual cwd from the first session file
                    if let Ok(sessions) = fs::read_dir(entry.path()) {
                        for session in sessions.flatten() {
                            if session.path().extension().and_then(|s| s.to_str()) == Some("jsonl") {
                                if let Some(found_cwd) = extract_cwd_from_file(&session.path()) {
                                    cwd = Some(found_cwd);
                                    break;
                                }
                            }
                        }
                    }

                    let name = cwd.as_ref()
                        .and_then(|p| StdPath::new(p).file_name())
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| id.replace("-", "/"));

                    projects.push(Project {
                        id,
                        name,
                        cwd,
                    });
                }
            }
        }
    }
    
    axum::Json(projects)
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

async fn list_sessions(Path(project_id): Path<String>) -> axum::Json<Vec<Session>> {
    let path = get_projects_path().join(&project_id);
    let mut sessions = Vec::new();
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                let id = entry.file_name().to_string_lossy().into_owned();
                let title = get_session_title(&file_path).unwrap_or_else(|| id.clone());
                sessions.push(Session {
                    id,
                    project_id: project_id.clone(),
                    title,
                    timestamp: None,
                });
            }
        }
    }
    
    axum::Json(sessions)
}

async fn recent_sessions() -> axum::Json<Vec<Session>> {
    let path = get_projects_path();
    let mut all_sessions = Vec::new();

    if let Ok(project_entries) = fs::read_dir(path) {
        for project_entry in project_entries.flatten() {
            if project_entry.path().is_dir() {
                let project_id = project_entry.file_name().to_string_lossy().into_owned();
                if let Ok(session_entries) = fs::read_dir(project_entry.path()) {
                    for session_entry in session_entries.flatten() {
                        let file_path = session_entry.path();
                        if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                            let mtime = fs::metadata(&file_path).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                            all_sessions.push((mtime, project_id.clone(), file_path));
                        }
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

    axum::Json(recent)
}

fn get_session_title(path: &std::path::Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(msg) = v.get("message") {
                if let Some(content) = msg.get("content") {
                    let text = if let Some(text) = content.as_str() {
                        text.to_string()
                    } else if let Some(arr) = content.as_array() {
                        arr.iter().filter_map(|item| item.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join(" ")
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

async fn read_session(Path((project_id, session_id)): Path<(String, String)>) -> impl IntoResponse {
    let path = get_projects_path().join(&project_id).join(&session_id);
    
    match fs::read_to_string(path) {
        Ok(content) => Response::new(Body::from(content)),
        Err(_) => Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not Found")).unwrap(),
    }
}

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    
    if path.is_empty() || path == "index.html" {
        return index_handler();
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => index_handler(),
    }
}

fn index_handler() -> Response {
    match Assets::get("index.html") {
        Some(content) => Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content.data))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}
