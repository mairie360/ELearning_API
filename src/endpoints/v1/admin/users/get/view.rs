use utoipa::ToSchema;

/// Active user of the platform.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the user.
    #[schema(example = 42)]
    pub id: u64,
    /// First and last name of the user.
    #[schema(example = "Jean Dupont")]
    pub name: String,
}

/// One page of the active users.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersResultView {
    /// Active users of the page, in `id` order. Empty past the last page.
    pub users: Vec<User>,
}
