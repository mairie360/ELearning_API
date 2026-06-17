use actix_web::web;

use crate::endpoints::v1::admin::formations::formation_id::register::endpoint::RegisterUserToFormationError;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterUserView {
    user_id: u64,
}

impl RegisterUserView {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl TryFrom<web::Json<RegisterUserView>> for RegisterUserView {
    type Error = RegisterUserToFormationError;

    fn try_from(params: web::Json<RegisterUserView>) -> Result<RegisterUserView, Self::Error> {
        Ok(params.into_inner())
    }
}
