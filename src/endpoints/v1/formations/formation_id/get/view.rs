use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Module {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetFormationResponseView {
    pub modules: Vec<Module>,
}
