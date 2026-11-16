/// Progress of the signed-in user in a formation: `NotStarted`, `InProgress` or `Completed`.
/// `Error` flags a stored value the API cannot interpret.
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

/// Formation the signed-in user is enrolled in.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct Formation {
    /// Id of the formation, to reuse in `/api/v1/formations/{formation_id}/`.
    #[schema(example = 4)]
    id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    name: String,
    /// Description of the formation. Empty string if it has none — never `null`.
    #[schema(example = "Obligations and good practices")]
    description: String,
    /// Progress of the signed-in user. `Error` flags a stored value the API cannot
    /// interpret.
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

/// Formations of the signed-in user.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetFormationsResultView {
    /// Formations the signed-in user is enrolled in. Empty if they follow none.
    formations: Vec<Formation>,
}

impl GetFormationsResultView {
    pub fn new(formations: Vec<Formation>) -> Self {
        GetFormationsResultView { formations }
    }
}
