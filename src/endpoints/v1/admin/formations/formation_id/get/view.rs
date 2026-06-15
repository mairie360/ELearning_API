use crate::endpoints::v1::admin::formations::AdminFormationModule;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationByIdResultView {
    pub modules: Vec<AdminFormationModule>,
}
