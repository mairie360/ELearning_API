use crate::endpoints::v1::admin::users::UsersFormation;

/// Formations an agent is enrolled in.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserByIdResultView {
    /// Formations the agent is enrolled in. Empty if they follow none, or if the id
    /// matches nobody.
    pub formations: Vec<UsersFormation>,
}
