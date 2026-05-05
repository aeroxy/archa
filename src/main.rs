use axum::{routing::get, Router, extract::{Path, State}, response::{IntoResponse, Response}, http::{header, StatusCode, Uri}, body::Body};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use rust_embed::Embed;
use std::path::PathBuf;
use tokio::net::TcpListener;
use std::sync::Arc;
use clap::Parser;

mod model;
mod claude;
mod opencode;
mod backend;

use backend::{AppState, Backend};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Custom path to Claude projects
    #[arg(short = 'd', long)]
    projects_path: Option<PathBuf>,
}

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Assets;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new(args.projects_path.clone()));

    let app = Router::new()
        .route("/api/_/backends", get(list_backends))
        .route("/api/{cli}/projects", get(list_projects))
        .route("/api/{cli}/sessions/{project_id}", get(list_sessions))
        .route("/api/{cli}/session/{project_id}/{session_id}", get(read_session))
        .route("/api/{cli}/recent-sessions", get(recent_sessions))
        .route("/api/{cli}/session-info/{session_id}", get(get_session_project))
        .fallback(static_handler)
        .with_state(state)
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

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

async fn list_backends(State(state): State<Arc<AppState>>) -> axum::Json<Vec<String>> {
    axum::Json(state.backend_ids())
}

async fn list_projects(
    Path(cli): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(backend) = Backend::from_cli(&cli, &state) else { return not_found() };
    axum::Json(backend.list_projects()).into_response()
}

async fn list_sessions(
    Path((cli, project_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(backend) = Backend::from_cli(&cli, &state) else { return not_found() };
    axum::Json(backend.list_sessions(&project_id)).into_response()
}

async fn recent_sessions(
    Path(cli): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(backend) = Backend::from_cli(&cli, &state) else { return not_found() };
    axum::Json(backend.recent_sessions()).into_response()
}

async fn get_session_project(
    Path((cli, session_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(backend) = Backend::from_cli(&cli, &state) else { return not_found() };
    match backend.find_session(&session_id) {
        Some(info) => axum::Json(info).into_response(),
        None => not_found(),
    }
}

async fn read_session(
    Path((cli, project_id, session_id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let Some(backend) = Backend::from_cli(&cli, &state) else { return not_found() };
    match backend.read_session(&project_id, &session_id) {
        Some(content) => Response::new(Body::from(content)),
        None => not_found(),
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
