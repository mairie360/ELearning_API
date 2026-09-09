use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Registers a user to a course (`user_courses`, defaulting to
/// `not_started`). Idempotent: registering the same pair twice is a no-op.
/// The caller is expected to have already checked that the user and the
/// course exist (see `does_user_exist` in the lib and
/// `formations::does_course_exist` here) so a foreign-key violation here
/// would only mean a race, not routine input validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterUserToFormationQueryView {
    params: Vec<QueryParam>,
}

impl RegisterUserToFormationQueryView {
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

impl ApiRequestDto for RegisterUserToFormationQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO user_courses (user_id, course_id) \
         VALUES ($1, $2) \
         ON CONFLICT (user_id, course_id) DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
