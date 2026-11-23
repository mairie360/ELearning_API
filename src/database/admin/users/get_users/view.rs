use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};

/// Lists one page of the active (non-archived) users in `id` order, for the
/// admin "assign to a formation" picker. The caller bounds `limit`.
///
/// Two optional filters (MAIR-817), bound as values that mean "no filter" since `QueryParam` has
/// no optional type:
/// - `search` (`""` for none): case-insensitive substring of `first last`, `last first` or the
///   e-mail. The expression is the one of Database's trigram index `idx_users_search_text_trgm`
///   (`mair-477-02`), written exactly so the index serves it; `%`, `_` and `\` of the term are
///   escaped, so they match themselves.
/// - `exclude_formation_id` (`0` for none): leaves out the users enrolled in that formation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUsersQueryView {
    params: Vec<QueryParam>,
}

impl GetUsersQueryView {
    /// Every active user, one page.
    pub fn new(limit: u32, offset: u32) -> Self {
        Self::filtered(limit, offset, None, None)
    }

    /// One page of the active users matching `search` and not enrolled in
    /// `exclude_formation_id`. The caller trims and bounds `search`.
    pub fn filtered(
        limit: u32,
        offset: u32,
        search: Option<&str>,
        exclude_formation_id: Option<u64>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
                QueryParam::Text(search.unwrap_or_default().to_string()),
                QueryParam::I32(exclude_formation_id.map_or(0, id_to_sql)),
            ],
        }
    }
}

impl ApiRequestDto for GetUsersQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT u.id, (u.first_name || ' ' || u.last_name) AS name, \
                u.first_name, u.last_name \
            FROM users u \
            WHERE u.is_archived = FALSE \
                AND ($3 = '' OR lower(u.first_name || ' ' || u.last_name || chr(31) \
                    || u.last_name || ' ' || u.first_name || chr(31) || u.email) \
                    LIKE '%' || replace(replace(replace(lower($3), '\\', '\\\\'), \
                        '%', '\\%'), '_', '\\_') || '%') \
                AND ($4 = 0 OR NOT EXISTS ( \
                    SELECT 1 FROM user_courses uc \
                    WHERE uc.user_id = u.id AND uc.course_id = $4)) \
            ORDER BY u.id \
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
    first_name: String,
    last_name: String,
}

impl UserRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }
}
