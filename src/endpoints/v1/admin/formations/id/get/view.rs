use crate::endpoints::v1::admin::formations::AdminFormationModule;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationByIdResultView {
    formation: Vec<AdminFormationModule>,
}

impl GetFormationByIdResultView {
    pub fn new(formation: Vec<AdminFormationModule>) -> Self {
        Self { formation }
    }

    pub fn formation(&self) -> &[AdminFormationModule] {
        &self.formation
    }
}
