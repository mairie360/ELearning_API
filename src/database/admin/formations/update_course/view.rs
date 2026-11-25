use crate::database::admin::stats::get_stats::view::STATS_CACHE_NAMESPACE;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::admin::formations::create_course::view::CourseFields;

/// Replaces the fields of a course with [`CourseFields`]. A course whose instructor is an account
/// or a group (set outside the API) keeps it unless an `instructor` label replaces it (the schema
/// allows only one of the three). `fetch_scalar` reads whether the course
/// exists (`false`: nothing was written). The update locks the course row, so the rest of the
/// transaction of `PATCH /admin/formations/{formation_id}/` runs alone on that course.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateCourseQueryView {
    params: Vec<QueryParam>,
}

impl UpdateCourseQueryView {
    pub fn new(formation_id: u64, fields: &CourseFields) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(formation_id)), fields.to_param()],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for UpdateCourseQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH updated AS ( \
            UPDATE courses SET title = j->>'name', description = j->>'description', \
                category = j->>'category', level = j->>'level', \
                is_mandatory = (j->>'is_mandatory')::boolean, deadline = (j->>'deadline')::date, \
                source = j->>'source', instructor_label = j->>'instructor', \
                instructor_user_id = CASE WHEN j->>'instructor' IS NULL \
                    THEN instructor_user_id END, \
                instructor_group_id = CASE WHEN j->>'instructor' IS NULL \
                    THEN instructor_group_id END \
            FROM (SELECT $2::jsonb AS j) input \
            WHERE id = $1 \
            RETURNING id \
         ) \
         SELECT EXISTS(SELECT 1 FROM updated)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        &[STATS_CACHE_NAMESPACE]
    }
}
