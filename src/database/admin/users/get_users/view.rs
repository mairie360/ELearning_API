use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists one page of the active (non-archived) users in `id` order, for the
/// admin "assign to a formation" picker. The caller bounds `limit`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUsersQueryView {
    params: Vec<QueryParam>,
}

impl GetUsersQueryView {
    pub fn new(limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }
}

impl ApiRequestDto for GetUsersQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT id, (first_name || ' ' || last_name) AS name \
            FROM users \
            WHERE is_archived = FALSE \
            ORDER BY id \
            LIMIT $1 OFFSET $2 \
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
