pub mod doc;
pub mod get;
pub mod user_id;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Completed,
    Error,
}

impl From<String> for ProgressStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "not_started" => ProgressStatus::NotStarted,
            "in_progress" => ProgressStatus::InProgress,
            "completed" => ProgressStatus::Completed,
            _ => ProgressStatus::Error,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersFormation {
    /// Identifiant de la formation.
    #[schema(example = 4)]
    id: u64,
    /// Intitulé de la formation.
    #[schema(example = "RGPD pour les agents territoriaux")]
    name: String,
    /// Description de la formation.
    #[schema(example = "Obligations et bonnes pratiques")]
    description: String,
    /// Modules de la formation, ou `null` si `details` n'a pas été demandé.
    modules: Option<Vec<UsersFormationModule>>,
    // `user_courses.started_at` has no default and is only set by the
    // `fn_update_user_course_progress` trigger once the user completes their
    // first module, so a freshly-registered formation has none yet.
    /// Date du premier module terminé, ou `null` si l'agent n'en a terminé aucun. Posée par un
    /// déclencheur en base, pas à l'inscription.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-02T08:30:00Z")]
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Date d'achèvement de la formation entière, ou `null` si elle n'est pas terminée.
    #[schema(value_type = Option<String>, format = DateTime)]
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Avancement de l'agent sur cette formation, recalculé automatiquement en base.
    progress_status: ProgressStatus,
}

impl UsersFormation {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        modules: Option<Vec<UsersFormationModule>>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        progress_status: ProgressStatus,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            modules,
            started_at,
            completed_at,
            progress_status,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn modules(&self) -> &Option<Vec<UsersFormationModule>> {
        &self.modules
    }

    pub fn started_at(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.started_at
    }

    pub fn completed_at(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.completed_at
    }

    pub fn progress_status(&self) -> &ProgressStatus {
        &self.progress_status
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersFormationModule {
    /// Identifiant du module.
    #[schema(example = 11)]
    id: u64,
    /// Intitulé du module.
    #[schema(example = "Les principes du RGPD")]
    name: String,
    /// Description du module.
    #[schema(example = "Licéité, minimisation, durée de conservation")]
    description: String,
    /// Pièces jointes du module, avec la date à laquelle l'agent les a consultées.
    content: Vec<UsersModuleContent>,
    /// `true` si l'agent a terminé ce module.
    #[schema(example = true)]
    is_completed: bool,
    /// Date d'achèvement du module par l'agent, ou `null` s'il ne l'a pas terminé.
    #[schema(value_type = String, format = DateTime, example = "2026-09-03T14:25:00Z")]
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UsersFormationModule {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        content: Vec<UsersModuleContent>,
        is_completed: bool,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            content,
            is_completed,
            completed_at,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn content(&self) -> &Vec<UsersModuleContent> {
        &self.content
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersModuleContent {
    /// Identifiant de la pièce jointe.
    #[schema(example = 31)]
    id: u64,
    /// Nom du fichier.
    #[schema(example = "rgpd-principes.pdf")]
    file_name: String,
    /// Type du fichier, tel qu'il est stocké en base : `video` ou `pdf`.
    #[schema(example = "pdf")]
    file_type: String,
    /// Date à laquelle l'agent a terminé cette pièce jointe, ou `null` s'il ne l'a pas encore
    /// consultée.
    #[schema(value_type = String, format = DateTime, example = "2026-09-03T14:22:00Z")]
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UsersModuleContent {
    pub fn new(
        id: u64,
        file_name: &str,
        file_type: &str,
        finished_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            file_name: file_name.to_string(),
            file_type: file_type.to_string(),
            finished_at,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }

    pub fn finished_at(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.finished_at
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/users")
            .service(get::endpoint::get_users)
            .configure(user_id::config),
    );
}
