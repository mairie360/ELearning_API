#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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
