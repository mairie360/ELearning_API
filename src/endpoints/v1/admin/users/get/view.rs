use utoipa::ToSchema;

/// Agent inscrit à au moins une formation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Identifiant Core API de l'agent.
    #[schema(example = 42)]
    pub id: u64,
    /// Prénom et nom de l'agent.
    #[schema(example = "Jean Dupont")]
    pub name: String,
}

/// Agents inscrits à au moins une formation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersResultView {
    /// Agents ayant au moins une inscription. Ce n'est pas l'annuaire complet de la plateforme.
    pub users: Vec<User>,
}
