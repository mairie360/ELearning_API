use crate::endpoints::validation::{check_user_id, Validate, ValidationError};

/// Utilisateur à inscrire à la formation du chemin.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterUserView {
    /// Identifiant Core API de l'agent à inscrire, tel que le renvoie `GET /api/v1/user/`
    /// de Core API.
    #[schema(example = 42)]
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

impl Validate for RegisterUserView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_user_id("user_id", self.user_id)
    }
}
