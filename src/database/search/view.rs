use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

/// What a global search result is. ELearning finds formations only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalSearchKind {
    /// A formation; `id` is reusable in `GET /api/v1/formations/{formation_id}/`.
    Course,
}

/// One result of the global search.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct GlobalSearchItem {
    pub kind: GlobalSearchKind,
    /// Id of the formation.
    #[schema(example = 3)]
    pub id: i64,
    /// Title of the formation.
    #[schema(example = "Welcoming the public")]
    pub title: String,
    /// First 120 characters of the description, `null` without one.
    #[schema(example = "Front-desk practices for the town hall services")]
    pub excerpt: Option<String>,
    /// Always `null` for a formation (no parent item).
    #[schema(example = json!(null))]
    pub parent_id: Option<i64>,
    /// Creation of the formation (UTC, RFC 3339): formations carry no modification date.
    #[schema(format = DateTime, example = "2026-09-01T08:00:00Z")]
    pub updated_at: String,
}

/// Formations matching a global search (`GET /api/v1/search`), among those of the caller's
/// catalogue: the formations they are enrolled in, every formation for an administrator. A
/// formation matches on its title through the trigram index. Titles starting with the term come
/// first, then the most recent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchQueryView {
    params: Vec<QueryParam>,
}

impl GlobalSearchQueryView {
    /// `pattern` is the term already escaped for `LIKE` (see `like_escape`), `limit` bounded by
    /// the endpoint.
    pub fn new(user_id: u64, pattern: &str, limit: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(pattern.to_string()),
                QueryParam::I64(i64::try_from(limit).unwrap_or(i64::MAX)),
            ],
        }
    }
}

impl ApiRequestDto for GlobalSearchQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT jsonb_build_object( \
            'kind', 'course', 'id', c.id, 'title', c.title, \
            'excerpt', NULLIF(left(btrim(COALESCE(c.description, '')), 120), ''), \
            'parent_id', NULL, \
            'updated_at', to_char(c.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')) \
         FROM courses c \
         WHERE (c.title ILIKE '%' || $2 || '%' OR COALESCE(c.description, '') ILIKE '%' || $2 || '%') \
           AND ((SELECT is_admin($1)) \
                OR EXISTS (SELECT 1 FROM user_courses uc \
                           WHERE uc.course_id = c.id AND uc.user_id = $1)) \
         ORDER BY (c.title ILIKE $2 || '%') DESC, c.created_at DESC NULLS LAST, c.id DESC \
         LIMIT $3"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

impl Display for GlobalSearchQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The search term is not logged.
        write!(f, "GlobalSearchQueryView: limit={:?}", self.params[2])
    }
}
