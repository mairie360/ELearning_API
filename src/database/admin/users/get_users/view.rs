use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists active (non-archived) users, for the admin "assign to a formation"
/// picker.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GetUsersQueryView {
    params: Vec<QueryParam>,
}

impl GetUsersQueryView {
    pub fn new() -> Self {
        Self { params: vec![] }
    }
}

impl ApiRequestDto for GetUsersQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT id, (first_name || ' ' || last_name) AS name \
            FROM users \
            WHERE is_archived = FALSE \
            ORDER BY id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserRow {
    id: i32,
    name: String,
}

impl UserRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
