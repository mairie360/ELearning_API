use crate::endpoints::v1::admin::formations::AdminFormationModule;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationByIdResultView {
    /// Modules de la formation, dans l'ordre du cours.
    pub modules: Vec<AdminFormationModule>,
}
