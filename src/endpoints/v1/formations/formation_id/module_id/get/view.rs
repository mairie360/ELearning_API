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

impl ToString for FileType {
    fn to_string(&self) -> String {
        match self {
            FileType::Video => "video".to_string(),
            FileType::Pdf => "pdf".to_string(),
            FileType::Error => "error".to_string(),
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
