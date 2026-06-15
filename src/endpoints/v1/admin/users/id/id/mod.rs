pub mod delete;
pub mod doc;
pub mod get;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminUserFormationIdParams {
    user_id: u64,
    formation_id: u64,
}

impl AdminUserFormationIdParams {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}
