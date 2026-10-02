pub mod doc;
pub mod get;
pub mod user_id;

/// Page size of `GET /api/v1/admin/users/` when `limit` is absent.
pub const DEFAULT_USERS_PAGE_SIZE: u32 = 50;
/// Largest page `GET /api/v1/admin/users/` returns, whatever `limit` asks for.
pub const MAX_USERS_PAGE_SIZE: u32 = 200;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminUsersPageQuery {
    /// Maximum number of users to return. Defaults to 50; values above 200 are capped to 200
    /// and `0` is raised to 1.
    #[param(example = 50, minimum = 1, maximum = 200)]
    limit: Option<u32>,
    /// Number of users to skip, in `id` order. Defaults to 0. Page `n` (from 0) is
    /// `offset = n * limit`; a page shorter than `limit` is the last one.
    #[param(example = 0, minimum = 0)]
    offset: Option<u32>,
}

impl AdminUsersPageQuery {
    /// The page size actually used, within `1..=MAX_USERS_PAGE_SIZE`.
    pub fn limit(&self) -> u32 {
        self.limit
            .unwrap_or(DEFAULT_USERS_PAGE_SIZE)
            .clamp(1, MAX_USERS_PAGE_SIZE)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

/// Avancement d'un agent dans une formation : `NotStarted`, `InProgress` ou `Completed`.
/// `Error` signale une valeur en base que l'API ne sait pas interpréter.
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

/// Formation à laquelle un agent est inscrit, avec son avancement.
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
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-20T16:45:00Z")]
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

/// Module d'une formation et son avancement pour l'agent.
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

/// Pièce jointe d'un module et sa date de consultation par l'agent.
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
