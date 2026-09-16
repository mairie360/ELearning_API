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
    /// Identifiant de la formation, à réutiliser dans `/api/v1/formations/{formation_id}/`.
    #[schema(example = 4)]
    id: u64,
    /// Intitulé de la formation.
    #[schema(example = "RGPD pour les agents territoriaux")]
    name: String,
    /// Description de la formation. Chaîne vide s'il n'y en a pas — jamais `null`.
    #[schema(example = "Obligations et bonnes pratiques")]
    description: String,
    /// Avancement de l'utilisateur connecté. `Error` signale une valeur en base que l'API ne
    /// sait pas interpréter.
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
    /// Formations auxquelles l'utilisateur connecté est inscrit. Vide s'il n'en suit aucune.
    formations: Vec<Formation>,
}

impl GetFormationsResultView {
    pub fn new(formations: Vec<Formation>) -> Self {
        GetFormationsResultView { formations }
    }
}
