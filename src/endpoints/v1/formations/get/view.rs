#[derive(Debug, Default, serde::Serialize, utoipa::ToSchema)]
pub enum Status {
    Completed,
    InProgress,
    NotStarted,
    #[default]
    Error,
}

impl From<String> for Status {
    fn from(s: String) -> Self {
        match s.as_str() {
            "completed" => Status::Completed,
            "in_progress" => Status::InProgress,
            "not_started" => Status::NotStarted,
            _ => Status::Error,
        }
    }
}

impl From<Status> for String {
    fn from(status: Status) -> Self {
        match status {
            Status::Completed => "completed".to_string(),
            Status::InProgress => "in_progress".to_string(),
            Status::NotStarted => "not_started".to_string(),
            Status::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct Formation {
    id: u64,
    name: String,
    description: String,
    status: Status,
}

impl Formation {
    pub fn new(id: u64, name: &str, description: &str, status: Status) -> Self {
        Formation {
            id,
            name: name.to_string(),
            description: description.to_string(),
            status,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetFormationsResultView {
    formations: Vec<Formation>,
}

impl GetFormationsResultView {
    pub fn new(formations: Vec<Formation>) -> Self {
        GetFormationsResultView { formations }
    }
}
