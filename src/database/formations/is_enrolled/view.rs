use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

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
                QueryParam::I32(user_id as i32),
                QueryParam::I32(formation_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[1].as_i32() as u64
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
