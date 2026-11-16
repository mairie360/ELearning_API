use crate::endpoints::v1::admin::formations::AdminFormation;
use utoipa::ToSchema;

/// Catalogue of the formations, admin view.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationsResultView {
    /// Formations of the requested page (`limit` / `offset`), not the whole catalogue.
    pub formations: Vec<AdminFormation>,
}
