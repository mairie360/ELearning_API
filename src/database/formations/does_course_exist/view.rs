use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Existence check for a `courses` row, mirroring the lib's own
/// `DoesUserExistByIdQueryView`. Used to turn a missing formation into a 404
/// instead of a silently empty result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoesCourseExistQueryView {
    params: Vec<QueryParam>,
}

impl DoesCourseExistQueryView {
    pub fn new(course_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(course_id))],
        }
    }

    pub fn course_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for DoesCourseExistQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1) AS does_course_exist"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
