use utoipa::ToSchema;

use crate::endpoints::validation::{check_label, check_user_id, Validate, ValidationError};

/// Longest `search` term, once trimmed.
pub const MAX_SEARCH_LENGTH: usize = 100;

/// Optional filters of `GET /api/v1/admin/users/` (MAIR-817).
#[derive(Debug, Default, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminUsersFilterQuery {
    /// Keeps the users whose `first last` name, `last first` name or e-mail contains this term,
    /// whatever the letter case. Trimmed; 1 to 100 characters, no control character.
    #[param(example = "dupont", min_length = 1, max_length = 100)]
    search: Option<String>,
    /// Leaves out the users already enrolled in this formation (the « add agents » picker of a
    /// formation). An id that matches no formation leaves nobody out.
    #[param(example = 4, minimum = 1, maximum = 2147483647)]
    exclude_formation_id: Option<u64>,
}

impl AdminUsersFilterQuery {
    pub fn new(search: Option<&str>, exclude_formation_id: Option<u64>) -> Self {
        Self {
            search: search.map(str::to_string),
            exclude_formation_id,
        }
    }

    /// The search term, trimmed.
    pub fn search(&self) -> Option<&str> {
        self.search.as_deref().map(str::trim)
    }

    pub fn exclude_formation_id(&self) -> Option<u64> {
        self.exclude_formation_id
    }
}

impl Validate for AdminUsersFilterQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(search) = self.search() {
            check_label("search", search, MAX_SEARCH_LENGTH)?;
        }
        if let Some(id) = self.exclude_formation_id {
            check_user_id("exclude_formation_id", id)?;
        }
        Ok(())
    }
}

/// Active user of the platform.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the user.
    #[schema(example = 42)]
    pub id: u64,
    /// First and last name of the user.
    #[schema(example = "Jean Dupont")]
    pub name: String,
    /// First name of the user.
    #[schema(example = "Jean")]
    pub first_name: String,
    /// Last name of the user.
    #[schema(example = "Dupont")]
    pub last_name: String,
}

/// One page of the active users.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersResultView {
    /// Active users of the page, in `id` order. Empty past the last page.
    pub users: Vec<User>,
}
