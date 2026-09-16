use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Module {
    /// Identifiant du module, à réutiliser dans `/api/v1/formations/{formation_id}/{module_id}/`.
    #[schema(example = 11)]
    pub id: u64,
    /// Intitulé du module.
    #[schema(example = "Les principes du RGPD")]
    pub name: String,
    /// Description du module.
    #[schema(example = "Licéité, minimisation, durée de conservation")]
    pub description: String,
    /// `true` si l'utilisateur connecté a terminé ce module.
    #[schema(example = true)]
    pub completed: bool,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationResponseView {
    /// Modules de la formation. Vide si l'appelant n'est pas inscrit à cette formation.
    pub modules: Vec<Module>,
}
