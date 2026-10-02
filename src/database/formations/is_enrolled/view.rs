use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Whether a user is enrolled in a course (`user_courses` row). Guards
/// `GET /v1/formations/{formation_id}/`. Never cached: an unenrolment must
/// take effect on the next request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsEnrolledQueryView {
    params: Vec<QueryParam>,
}

impl IsEnrolledQueryView {
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

impl ApiRequestDto for IsEnrolledQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT EXISTS(SELECT 1 FROM user_courses WHERE user_id = $1 AND course_id = $2) \
         AS is_enrolled"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
