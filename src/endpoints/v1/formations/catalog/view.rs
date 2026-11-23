use utoipa::ToSchema;

use crate::endpoints::v1::formations::formation_id::module_id::get::view::FileType;
use crate::endpoints::v1::formations::get::view::Status;

/// Level of a formation, as an administrator set it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CourseLevel {
    Beginner,
    Intermediate,
    Advanced,
}

impl CourseLevel {
    /// The level stored in `courses.level`; `None` for any other value.
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "beginner" => Some(Self::Beginner),
            "intermediate" => Some(Self::Intermediate),
            "advanced" => Some(Self::Advanced),
            _ => None,
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Advanced => "advanced",
        }
    }
}

/// Origin of a formation: `internal` (made by the mairie) or `platform` (provided with Mairie 360).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum CourseSource {
    #[default]
    Internal,
    Platform,
}

impl CourseSource {
    /// The origin stored in `courses.source`; anything unknown reads as `internal`, the default.
    pub fn from_db(value: &str) -> Self {
        if value == "platform" {
            Self::Platform
        } else {
            Self::Internal
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::Platform => "platform",
        }
    }
}

/// Type of a content, as stored: `video`, `pdf`, `document`, `link`, `quiz`, `audio` or `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Video,
    Pdf,
    Document,
    Link,
    Quiz,
    Audio,
    Other,
}

impl ContentKind {
    /// The `attachment_type` stored in `course_attachments.file_type`; `other` for an unknown one.
    pub fn from_db(value: &str) -> Self {
        match value {
            "video" => Self::Video,
            "pdf" => Self::Pdf,
            "document" => Self::Document,
            "link" => Self::Link,
            "quiz" => Self::Quiz,
            "audio" => Self::Audio,
            _ => Self::Other,
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Pdf => "pdf",
            Self::Document => "document",
            Self::Link => "link",
            Self::Quiz => "quiz",
            Self::Audio => "audio",
            Self::Other => "other",
        }
    }

    /// The types whose content is a file of the storage (an S3 object key in `file_url`): they
    /// can only be created by an upload, which the API does not offer yet.
    pub fn is_stored_file(self) -> bool {
        matches!(self, Self::Video | Self::Pdf | Self::Document | Self::Audio)
    }
}

/// File (content) of a module of the catalogue.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CatalogFile {
    /// Id of the content, to pass to
    /// `/api/v1/formations/{formation_id}/{module_id}/{attachment_id}/` to get the URL of a stored file.
    #[schema(example = 31)]
    pub id: u64,
    /// Name of the file (the title for a content without file).
    #[schema(example = "gdpr-principles.pdf")]
    pub file_name: String,
    /// Type of the file, `Video` or `Pdf`; `Error` for every other type (read `kind`).
    pub file_type: FileType,
    /// Size in bytes, or `null` if it was not recorded.
    #[schema(example = 482913)]
    pub file_size_bytes: Option<i64>,
    /// Exact type of the content.
    pub kind: ContentKind,
    /// Title shown to the agent.
    #[schema(example = "The principles of the GDPR")]
    pub title: String,
    /// Description of the content, `null` when there is none.
    #[schema(example = "Ten-minute summary")]
    pub description: Option<String>,
    /// Duration in seconds, `null` when it was not recorded.
    #[schema(example = 600)]
    pub duration_seconds: Option<u32>,
    /// `false` for an optional content.
    #[schema(example = true)]
    pub is_required: bool,
    /// Position of the content in its module, from 1.
    #[schema(example = 1)]
    pub sort_order: i32,
    /// URL of a `link` content (an absolute `https://` URL); `null` for every other type, whose
    /// file is opened through a signed URL.
    #[schema(example = json!(null))]
    pub url: Option<String>,
}

/// Module of a formation of the catalogue, with its files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CatalogModule {
    /// Module id, to reuse in `/api/v1/formations/{formation_id}/{module_id}/`.
    #[schema(example = 11)]
    pub id: u64,
    /// Title of the module.
    #[schema(example = "The principles of the GDPR")]
    pub name: String,
    /// Description of the module. Empty string when there is none — never `null`.
    #[schema(example = "Lawfulness, minimisation, retention period")]
    pub description: String,
    /// Position of the module in the formation, `null` when none was recorded.
    #[schema(example = 1)]
    pub sort_order: Option<i32>,
    /// `true` when the caller completed this module.
    #[schema(example = true)]
    pub completed: bool,
    /// Sum of the durations of its files, in seconds (0 when none has one).
    #[schema(example = 1800)]
    pub duration_seconds: u64,
    /// Files of the module, by `sort_order` then id. Empty when the module has none. Their
    /// contents are not returned: a signed URL is requested per stored file from
    /// `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.
    pub files: Vec<CatalogFile>,
}

/// Formation of the caller, with its modules and their files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CatalogFormation {
    /// Formation id, to reuse in `/api/v1/formations/{formation_id}/`.
    #[schema(example = 4)]
    pub id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    pub name: String,
    /// Description of the formation. Empty string when there is none — never `null`.
    #[schema(example = "Obligations and good practices")]
    pub description: String,
    /// Progress of the caller. `Error` is a value stored in the database that the API cannot
    /// interpret.
    pub status: Status,
    /// Category, `null` when none was set.
    #[schema(example = "Data protection")]
    pub category: Option<String>,
    /// Level, `null` when none was set.
    pub level: Option<CourseLevel>,
    /// `true` for a mandatory formation.
    #[schema(example = true)]
    pub is_mandatory: bool,
    /// Date by which the formation must be completed (`YYYY-MM-DD`), `null` without one.
    #[schema(value_type = Option<String>, format = Date, example = "2026-12-31")]
    pub deadline: Option<chrono::NaiveDate>,
    /// Origin of the formation.
    pub source: CourseSource,
    /// Instructor: the free label, else the name of the instructor account, `null` without one.
    #[schema(example = "Data protection officer")]
    pub instructor: Option<String>,
    /// Start of the caller on the formation (`user_courses.started_at`), `null` when the database
    /// recorded none.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-02T08:30:00Z")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completion of the formation by the caller, `null` while it is not completed.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Latest completed module of the caller on the formation, `null` before any.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-03T10:12:00Z")]
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Latest reminder an administrator sent the caller about this formation, `null` without one.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub last_reminder_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Average rating of the formation (1 to 5, one decimal), `null` while nobody rated it.
    #[schema(example = 4.5)]
    pub rating_average: Option<f64>,
    /// Number of ratings.
    #[schema(example = 12)]
    pub rating_count: u64,
    /// Rating the caller gave (1 to 5), `null` if they did not rate it.
    #[schema(example = json!(null))]
    pub my_rating: Option<u8>,
    /// Number of users enrolled in the formation.
    #[schema(example = 48)]
    pub learners: u64,
    /// Modules of the formation in their display order. Empty when the formation has none.
    pub modules: Vec<CatalogModule>,
}

/// Formations of the caller with their modules and files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetCatalogResultView {
    /// Formations the caller is enrolled in, by increasing id. Empty when they follow none.
    pub formations: Vec<CatalogFormation>,
}
