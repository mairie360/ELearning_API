use crate::endpoints::v1::admin::formations::AdminFormationModule;
use utoipa::ToSchema;

/// Modules of a formation, admin view.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationByIdResultView {
    /// Modules of the formation, in the order of the course.
    pub modules: Vec<AdminFormationModule>,
}
