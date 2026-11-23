use chrono::NaiveDateTime;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// The formations a user completed during a calendar year, with what their attestation states
/// (MAIR-1149): the user's name, the title and category of each formation, its completion date and
/// its training duration. One row per formation, by completion date then id.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMyCertificatesQueryView {
    params: Vec<QueryParam>,
}

impl GetMyCertificatesQueryView {
    pub fn new(user_id: u64, year: i32) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id)), QueryParam::I32(year)],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn year(&self) -> i32 {
        self.params[1].as_i32()
    }
}

impl ApiRequestDto for GetMyCertificatesQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT c.id, u.first_name, u.last_name, c.title, c.category, uc.completed_at, \
                COALESCE(( \
                    SELECT sum(ca.duration_seconds) FROM course_attachments ca \
                    JOIN course_modules cm ON cm.id = ca.module_id \
                    WHERE cm.course_id = c.id \
                ), 0)::bigint AS duration_seconds \
            FROM user_courses uc \
            JOIN users u ON u.id = uc.user_id \
            JOIN courses c ON c.id = uc.course_id \
            WHERE uc.user_id = $1 AND uc.status = 'completed' \
                AND uc.completed_at >= make_date($2, 1, 1) \
                AND uc.completed_at < make_date($2 + 1, 1, 1) \
            ORDER BY uc.completed_at, c.id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// One completed formation of a [`GetMyCertificatesQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompletedFormationRow {
    id: i32,
    first_name: String,
    last_name: String,
    title: String,
    category: Option<String>,
    completed_at: NaiveDateTime,
    duration_seconds: i64,
}

impl CompletedFormationRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn completed_at(&self) -> NaiveDateTime {
        self.completed_at
    }

    pub fn duration_seconds(&self) -> i64 {
        self.duration_seconds
    }
}
