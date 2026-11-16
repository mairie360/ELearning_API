pub mod doc;
pub mod formation_id;
pub mod get;

/// Formation of the catalogue, admin view.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminFormation {
    /// Id of the formation.
    #[schema(example = 4)]
    id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    name: String,
    /// Description of the formation.
    #[schema(example = "Obligations and good practices")]
    description: String,
    /// Modules of the formation, or `null` if `details` was not asked for. A `null` here means
    /// "not asked for", not "no module".
    modules: Option<Vec<AdminFormationModule>>,
}

impl AdminFormation {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        modules: Option<Vec<AdminFormationModule>>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            modules,
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

    pub fn modules(&self) -> Option<&Vec<AdminFormationModule>> {
        self.modules.as_ref()
    }
}

/// Module of a formation, admin view.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminFormationModule {
    /// Id of the module.
    #[schema(example = 11)]
    id: u64,
    /// Title of the module.
    #[schema(example = "The principles of the GDPR")]
    name: String,
    /// Description of the module.
    #[schema(example = "Lawfulness, minimisation, retention period")]
    description: String,
    /// Attachments of the module, or `null` if `details` was not asked for.
    content: Option<Vec<AdminModuleContent>>,
}

impl AdminFormationModule {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        content: Option<Vec<AdminModuleContent>>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            content,
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

    pub fn content(&self) -> Option<&Vec<AdminModuleContent>> {
        self.content.as_ref()
    }
}

/// Attachment of a module, admin view.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminModuleContent {
    /// Id of the attachment.
    #[schema(example = 31)]
    id: u64,
    /// Name of the file.
    #[schema(example = "rgpd-principes.pdf")]
    file_name: String,
    /// Type of the file, as stored in the database: `video` or `pdf`.
    #[schema(example = "pdf")]
    file_type: String,
}

impl AdminModuleContent {
    pub fn new(id: u64, file_name: &str, file_type: &str) -> Self {
        Self {
            id,
            file_name: file_name.to_string(),
            file_type: file_type.to_string(),
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
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/formations")
            .service(get::endpoint::get_formations)
            .configure(formation_id::config),
    );
}
