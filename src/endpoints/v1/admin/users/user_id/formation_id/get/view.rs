use crate::endpoints::v1::admin::users::UsersFormationModule;

/// Avancement d'un agent, module par module, dans une formation.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserFormation {
    /// Modules de la formation, avec la progression de l'agent sur chacun.
    pub modules: Vec<UsersFormationModule>,
}

impl GetUserFormation {
    pub fn new(modules: Vec<UsersFormationModule>) -> Self {
        Self { modules }
    }

    pub fn modules(&self) -> &[UsersFormationModule] {
        &self.modules
    }
}
