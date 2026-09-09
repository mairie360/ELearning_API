use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Unregisters a user from a course (`user_courses`). Cascades to that
/// user's per-module progress on the same course, since `user_modules`
/// references `course_modules` (not `user_courses`) and is left untouched;
/// re-registering the pair therefore starts a fresh `user_courses` row while
/// history in `user_modules` is preserved.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnsubUserFormationQueryView {
    params: Vec<QueryParam>,
}

impl UnsubUserFormationQueryView {
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

impl ApiRequestDto for UnsubUserFormationQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM user_courses WHERE user_id = $1 AND course_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
