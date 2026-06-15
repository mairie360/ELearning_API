use crate::endpoints::v1::admin::users::UsersFormation;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserByIdResultView {
    pub formations: Vec<UsersFormation>,
}
