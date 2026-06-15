pub mod doc;
pub mod get;
pub mod module_id;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct FormationIdParams {
    formation_id: u64,
}

impl FormationIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}
