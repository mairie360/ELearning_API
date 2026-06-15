use crate::endpoints::v1::admin::users::UsersFormation;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserByIdResultView {
    formations: Vec<UsersFormation>,
}

impl GetUserByIdResultView {
    pub fn new(formations: Vec<UsersFormation>) -> Self {
        Self { formations }
    }

    pub fn formations(&self) -> &[UsersFormation] {
        &self.formations
    }
}
