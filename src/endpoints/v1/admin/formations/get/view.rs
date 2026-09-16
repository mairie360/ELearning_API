use crate::endpoints::v1::admin::formations::AdminFormation;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationsResultView {
    /// Toutes les formations du catalogue.
    pub formations: Vec<AdminFormation>,
}
