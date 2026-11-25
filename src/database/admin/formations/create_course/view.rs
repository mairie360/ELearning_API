use crate::database::admin::stats::get_stats::view::STATS_CACHE_NAMESPACE;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// The fields of `courses` an administrator sets (MAIR-817), as the JSON object
/// [`CreateCourseQueryView`] and `UpdateCourseQueryView` bind: `name`, `description`, `category`,
/// `level`, `is_mandatory`, `deadline` (`YYYY-MM-DD`), `source` and `instructor` (stored in
/// `instructor_label`). Absent optional values are JSON `null`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CourseFields {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub level: Option<String>,
    pub is_mandatory: bool,
    pub deadline: Option<chrono::NaiveDate>,
    pub source: String,
    pub instructor: Option<String>,
}

impl CourseFields {
    /// The JSON text bound as a `jsonb` parameter.
    pub fn to_param(&self) -> QueryParam {
        QueryParam::Text(serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string()))
    }
}

/// Creates a course from its [`CourseFields`]; `fetch_scalar` reads its id.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateCourseQueryView {
    params: Vec<QueryParam>,
}

impl CreateCourseQueryView {
    pub fn new(fields: &CourseFields) -> Self {
        Self {
            params: vec![fields.to_param()],
        }
    }
}

impl ApiRequestDto for CreateCourseQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO courses (title, description, category, level, is_mandatory, deadline, \
            source, instructor_label) \
         SELECT j->>'name', j->>'description', j->>'category', j->>'level', \
            (j->>'is_mandatory')::boolean, (j->>'deadline')::date, j->>'source', \
            j->>'instructor' \
         FROM (SELECT $1::jsonb AS j) input \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        &[STATS_CACHE_NAMESPACE]
    }
}
