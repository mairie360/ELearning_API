use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Marks a module as completed for a user (upsert on the `(user_id, module_id)`
/// primary key), but only once the user has completed every module that comes
/// before it in its course (`sort_order`, then `id`, the order the listings
/// use; a `NULL` `sort_order` sorts last). A module the user already completed
/// is accepted again whatever the state of the previous ones (idempotence).
///
/// The order check and the upsert run as a single statement. Read the result
/// with `fetch_one` into a [`CompleteModuleRow`]. A `module_id` that doesn't
/// exist trips the `course_modules` foreign key and surfaces as
/// `DbError::ForeignKeyViolation` instead of silently doing nothing. The
/// database-side trigger (`fn_update_user_course_progress`) takes care of
/// rolling this up into the parent `user_courses` status.
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
        // A data-modifying CTE must sit at the top level, hence the `WITH`
        // around the usual `to_jsonb(t)` select.
        "WITH state AS ( \
            SELECT \
                EXISTS( \
                    SELECT 1 FROM user_modules \
                    WHERE user_id = $1 AND module_id = $2 AND is_completed \
                ) AS already_completed, \
                EXISTS( \
                    SELECT 1 \
                    FROM course_modules target \
                    JOIN course_modules prev ON prev.course_id = target.course_id \
                        AND (COALESCE(prev.sort_order, 2147483647), prev.id) \
                            < (COALESCE(target.sort_order, 2147483647), target.id) \
                    LEFT JOIN user_modules um \
                        ON um.module_id = prev.id AND um.user_id = $1 AND um.is_completed \
                    WHERE target.id = $2 AND um.module_id IS NULL \
                ) AS previous_pending \
         ), upserted AS ( \
            INSERT INTO user_modules (user_id, module_id, is_completed, completed_at) \
            SELECT $1, $2, TRUE, CURRENT_TIMESTAMP FROM state \
            WHERE already_completed OR NOT previous_pending \
            ON CONFLICT (user_id, module_id) \
            DO UPDATE SET is_completed = TRUE, completed_at = CURRENT_TIMESTAMP \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT (already_completed OR NOT previous_pending) AS completed FROM state \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of a [`CompleteModuleQueryView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompleteModuleRow {
    completed: bool,
}

impl CompleteModuleRow {
    /// `false` when a previous module of the course is not completed yet:
    /// nothing was written.
    pub fn completed(&self) -> bool {
        self.completed
    }
}
