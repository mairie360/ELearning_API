use utoipa::ToSchema;

/// Type d'une pièce jointe : `Video` ou `Pdf`.
/// `Error` couvre tout autre type stocké en base (`document`, `link`, `quiz`, `audio`, `other`), que l'API n'expose pas encore.
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

/// Pièce jointe d'un module.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct File {
    /// Identifiant de la pièce jointe, à passer à
    /// `/api/v1/formations/{formation_id}/{module_id}/{attachment_id}/` pour obtenir son URL.
    #[schema(example = 31)]
    pub id: u64,
    /// Nom du fichier tel qu'il sera présenté à l'agent.
    #[schema(example = "rgpd-principes.pdf")]
    pub file_name: String,
    /// Type du fichier. `Error` signale un type stocké en base que l'API ne sait pas interpréter.
    pub file_type: FileType,
    /// Taille en octets, ou `null` si elle n'a pas été enregistrée.
    #[schema(example = 482913)]
    pub file_size_bytes: Option<i64>,
}

/// Pièces jointes d'un module.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetModuleResponseView {
    /// Pièces jointes du module. Vide si le module n'en a aucune.
    pub files: Vec<File>,
}
