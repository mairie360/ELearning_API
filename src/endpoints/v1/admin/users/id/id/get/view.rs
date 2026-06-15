use crate::endpoints::v1::admin::users::UsersFormationModule;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserFormation {
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
