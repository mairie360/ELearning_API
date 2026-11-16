use crate::endpoints::v1::admin::formations::AdminFormation;
use utoipa::ToSchema;

/// Catalogue of the formations, admin view.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationsResultView {
    /// Every formation of the catalogue.
    pub formations: Vec<AdminFormation>,
}
