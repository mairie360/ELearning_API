pub mod doc;
pub mod get;
pub mod register;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminFormationIdParams {
    formation_id: u64,
}

impl AdminFormationIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}
