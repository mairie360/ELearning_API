pub mod doc;
pub mod get;
pub mod id;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminUserIdParams {
    user_id: u64,
}

impl AdminUserIdParams {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
