use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersResultView {
    id: u64,
    name: String,
}
