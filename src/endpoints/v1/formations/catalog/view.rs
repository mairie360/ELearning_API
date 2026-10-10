use utoipa::ToSchema;

use crate::endpoints::v1::formations::formation_id::module_id::get::view::File;
use crate::endpoints::v1::formations::get::view::Status;

/// Module of a formation of the catalogue, with its files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CatalogModule {
    /// Module id, to reuse in `/api/v1/formations/{formation_id}/{module_id}/`.
    #[schema(example = 11)]
    pub id: u64,
    /// Title of the module.
    #[schema(example = "Les principes du RGPD")]
    pub name: String,
    /// Description of the module. Empty string when there is none — never `null`.
    #[schema(example = "Licéité, minimisation, durée de conservation")]
    pub description: String,
    /// `true` when the caller completed this module.
    #[schema(example = true)]
    pub completed: bool,
    /// Files of the module (name, type, size), in the order of their ids. Empty when the module has
    /// none. Their contents are not returned: a signed URL is requested per file from
    /// `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.
    pub files: Vec<File>,
}

/// Formation of the caller, with its modules and their files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CatalogFormation {
    /// Formation id, to reuse in `/api/v1/formations/{formation_id}/`.
    #[schema(example = 4)]
    pub id: u64,
    /// Title of the formation.
    #[schema(example = "RGPD pour les agents territoriaux")]
    pub name: String,
    /// Description of the formation. Empty string when there is none — never `null`.
    #[schema(example = "Obligations et bonnes pratiques")]
    pub description: String,
    /// Progress of the caller. `Error` is a value stored in the database that the API cannot
    /// interpret.
    pub status: Status,
    /// Modules of the formation in their display order. Empty when the formation has none.
    pub modules: Vec<CatalogModule>,
}

/// Formations of the caller with their modules and files.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetCatalogResultView {
    /// Formations the caller is enrolled in, by increasing id. Empty when they follow none.
    pub formations: Vec<CatalogFormation>,
}
