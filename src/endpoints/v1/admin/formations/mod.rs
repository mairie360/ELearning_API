pub mod doc;
pub mod get;
pub mod id;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminFormation {
    id: u64,
    name: String,
    description: String,
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminFormationModule {
    id: u64,
    name: String,
    description: String,
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminModuleContent {
    id: u64,
    file_name: String,
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
