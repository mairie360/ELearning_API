use crate::endpoints::v1::admin::formations::AdminFormation;
use utoipa::ToSchema;

/// Catalogue des formations, vue administrateur.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationsResultView {
    /// Toutes les formations du catalogue.
    pub formations: Vec<AdminFormation>,
}
