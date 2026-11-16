pub mod doc;
pub mod get;
pub mod user_id;

/// Progress of an agent in a formation: `NotStarted`, `InProgress` or `Completed`.
/// `Error` flags a stored value the API cannot interpret.
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

/// Formation an agent is enrolled in, with their progress.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersFormation {
    /// Id of the formation.
    #[schema(example = 4)]
    id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    name: String,
    /// Description of the formation.
    #[schema(example = "Obligations and good practices")]
    description: String,
    /// Modules of the formation, or `null` if `details` was not asked for.
    modules: Option<Vec<UsersFormationModule>>,
    // `user_courses.started_at` has no default and is only set by the
    // `fn_update_user_course_progress` trigger once the user completes their
    // first module, so a freshly-registered formation has none yet.
    /// Date of the first completed module, or `null` if the agent completed none. Set by a
    /// database trigger, not on enrolment.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-02T08:30:00Z")]
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completion date of the whole formation, or `null` if it is not completed.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-20T16:45:00Z")]
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Progress of the agent on this formation, recomputed by the database.
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

/// Module of a formation and its progress for the agent.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersFormationModule {
    /// Id of the module.
    #[schema(example = 11)]
    id: u64,
    /// Title of the module.
    #[schema(example = "The principles of the GDPR")]
    name: String,
    /// Description of the module.
    #[schema(example = "Lawfulness, minimisation, retention period")]
    description: String,
    /// Attachments of the module, with the date the agent viewed them.
    content: Vec<UsersModuleContent>,
    /// `true` if the agent completed this module.
    #[schema(example = true)]
    is_completed: bool,
    /// Completion date of the module by the agent, or `null` if they did not complete it.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-03T14:25:00Z")]
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

/// Attachment of a module and the date the agent viewed it.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UsersModuleContent {
    /// Id of the attachment.
    #[schema(example = 31)]
    id: u64,
    /// Name of the file.
    #[schema(example = "rgpd-principes.pdf")]
    file_name: String,
    /// Type of the file, as stored in the database: `video` or `pdf`.
    #[schema(example = "pdf")]
    file_type: String,
    /// Date the agent completed this attachment, or `null` if they have not viewed it
    /// yet.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-03T14:22:00Z")]
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
