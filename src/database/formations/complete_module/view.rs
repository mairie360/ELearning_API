use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Marks a module as completed for a user (upsert on the `(user_id, module_id)`
/// primary key). A `module_id` that doesn't exist trips the `course_modules`
/// foreign key and surfaces as `DbError::ForeignKeyViolation` instead of
/// silently doing nothing. The database-side trigger
/// (`fn_update_user_course_progress`) takes care of rolling this up into the
/// parent `user_courses` status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompleteModuleQueryView {
    params: Vec<QueryParam>,
}

impl CompleteModuleQueryView {
    pub fn new(user_id: u64, module_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(module_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn module_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for CompleteModuleQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO user_modules (user_id, module_id, is_completed, completed_at) \
         VALUES ($1, $2, TRUE, CURRENT_TIMESTAMP) \
         ON CONFLICT (user_id, module_id) \
         DO UPDATE SET is_completed = TRUE, completed_at = CURRENT_TIMESTAMP"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
