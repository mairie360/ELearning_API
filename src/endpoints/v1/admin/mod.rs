pub mod doc;
pub mod formations;
pub mod users;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminUserDetailsQuery {
    #[serde(default)]
    details: bool,
}

impl AdminUserDetailsQuery {
    pub fn details(&self) -> bool {
        self.details
    }
}
