use crate::database::admin::stats::get_stats::view::STATS_CACHE_NAMESPACE;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Deletes a course (MAIR-817); its modules, files, enrolments, progress, ratings and reminders
/// go with it (`ON DELETE CASCADE`). `fetch_scalar` reads whether a course was deleted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteCourseQueryView {
    params: Vec<QueryParam>,
}

impl DeleteCourseQueryView {
    pub fn new(formation_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(formation_id))],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for DeleteCourseQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH deleted AS (DELETE FROM courses WHERE id = $1 RETURNING id) \
         SELECT EXISTS(SELECT 1 FROM deleted)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        &[STATS_CACHE_NAMESPACE]
    }
}
