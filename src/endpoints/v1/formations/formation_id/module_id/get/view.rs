use utoipa::ToSchema;

/// Type of an attachment: `Video` or `Pdf`.
/// `Error` covers any other type stored in the database (`document`, `link`, `quiz`, `audio`, `other`), which the API does not expose yet.
#[derive(Debug, serde::Serialize, ToSchema)]
pub enum FileType {
    Video,
    Pdf,
    Error,
}

impl From<String> for FileType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "video" => FileType::Video,
            "pdf" => FileType::Pdf,
            _ => FileType::Error,
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Video => write!(f, "video"),
            FileType::Pdf => write!(f, "pdf"),
            FileType::Error => write!(f, "error"),
        }
    }
}

/// Attachment of a module.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct File {
    /// Id of the attachment, to pass to
    /// `/api/v1/formations/{formation_id}/{module_id}/{attachment_id}/` to get its URL.
    #[schema(example = 31)]
    pub id: u64,
    /// Name of the file as shown to the agent.
    #[schema(example = "rgpd-principes.pdf")]
    pub file_name: String,
    /// Type of the file. `Error` flags a type stored in the database that the API cannot interpret.
    pub file_type: FileType,
    /// Size in bytes, or `null` if it was not recorded.
    #[schema(example = 482913)]
    pub file_size_bytes: Option<i64>,
}

/// Attachments of a module.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetModuleResponseView {
    /// Attachments of the module. Empty if the module has none.
    pub files: Vec<File>,
}
