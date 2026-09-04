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
    id: u64,
    name: String,
    description: String,
    modules: Option<Vec<UsersFormationModule>>,
    // `user_courses.started_at` has no default and is only set by the
    // `fn_update_user_course_progress` trigger once the user completes their
    // first module, so a freshly-registered formation has none yet.
    #[schema(value_type = Option<String>, format = DateTime)]
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
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
    id: u64,
    name: String,
    description: String,
    content: Vec<UsersModuleContent>,
    is_completed: bool,
    #[schema(value_type = String, format = DateTime)]
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
    id: u64,
    file_name: String,
    file_type: String,
    #[schema(value_type = String, format = DateTime)]
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
