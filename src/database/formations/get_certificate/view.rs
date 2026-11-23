use chrono::NaiveDateTime;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// What the attestation of one formation needs (MAIR-1149), in one statement: whether the user is
/// enrolled in it and completed it, the user's name, the title and category of the formation, the
/// completion date and the training duration (sum of the `duration_seconds` of its contents).
///
/// Read with `fetch_all`: one row for an existing user (the caller of a valid JWT), none
/// otherwise. An unknown formation reads as "not enrolled".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetCertificateQueryView {
    params: Vec<QueryParam>,
}

impl GetCertificateQueryView {
    pub fn new(user_id: u64, formation_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(formation_id)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for GetCertificateQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT (uc.user_id IS NOT NULL) AS enrolled, \
                COALESCE(uc.status = 'completed', FALSE) AS completed, \
                u.first_name, u.last_name, c.title, c.category, uc.completed_at, \
                COALESCE(( \
                    SELECT sum(ca.duration_seconds) FROM course_attachments ca \
                    JOIN course_modules cm ON cm.id = ca.module_id \
                    WHERE cm.course_id = $2 \
                ), 0)::bigint AS duration_seconds \
            FROM users u \
            LEFT JOIN user_courses uc ON uc.user_id = u.id AND uc.course_id = $2 \
            LEFT JOIN courses c ON c.id = uc.course_id \
            WHERE u.id = $1 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Result of a [`GetCertificateQueryView`]. The formation fields are `None` when the user is not
/// enrolled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CertificateRow {
    enrolled: bool,
    completed: bool,
    first_name: String,
    last_name: String,
    title: Option<String>,
    category: Option<String>,
    completed_at: Option<NaiveDateTime>,
    duration_seconds: i64,
}

impl CertificateRow {
    pub fn enrolled(&self) -> bool {
        self.enrolled
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn completed_at(&self) -> Option<NaiveDateTime> {
        self.completed_at
    }

    pub fn duration_seconds(&self) -> i64 {
        self.duration_seconds
    }
}
