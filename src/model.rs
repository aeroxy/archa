use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Project {
    pub name: String,
    pub id: String,
    pub cwd: Option<String>,
}

#[derive(Serialize)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub timestamp: Option<String>,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub project_id: String,
}
