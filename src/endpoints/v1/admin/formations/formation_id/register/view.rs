use crate::endpoints::validation::{check_user_id, Validate, ValidationError};

/// User to enrol in the formation of the path.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterUserView {
    /// Core API id of the agent to enrol, as returned by `GET /api/v1/user/`
    /// of Core API.
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
