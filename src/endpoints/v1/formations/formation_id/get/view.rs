use utoipa::ToSchema;

/// Module of a formation of the signed-in user, with its completion state.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Module {
    /// Id of the module, to reuse in `/api/v1/formations/{formation_id}/{module_id}/`.
    #[schema(example = 11)]
    pub id: u64,
    /// Title of the module.
    #[schema(example = "The principles of the GDPR")]
    pub name: String,
    /// Description du module.
    #[schema(example = "Lawfulness, minimisation, retention period")]
    pub description: String,
    /// `true` if the signed-in user completed this module.
    #[schema(example = true)]
    pub completed: bool,
}

/// Modules of a formation of the signed-in user.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationResponseView {
    /// Modules of the formation. Empty if the caller is not enrolled in this formation.
    pub modules: Vec<Module>,
}
