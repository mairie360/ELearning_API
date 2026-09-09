use utoipa::ToSchema;

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

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct File {
    pub id: u64,
    pub file_name: String,
    pub file_type: FileType,
    pub file_url: String,
    pub file_size_bytes: Option<i64>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetModuleResponseView {
    pub files: Vec<File>,
}
