use crate::endpoints::v1::admin::formations::AdminFormation;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationsResultView {
    formations: Vec<AdminFormation>,
}

impl GetFormationsResultView {
    pub fn new(formations: Vec<AdminFormation>) -> Self {
        Self { formations }
    }

    pub fn formations(&self) -> &[AdminFormation] {
        &self.formations
    }
}
