use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Marks a module as completed for a user (upsert on the `(user_id, module_id)`
/// primary key), but only when the user is enrolled in the course of the path,
/// the module belongs to that course, and the user has completed every module
/// that comes before it in the course (`sort_order`, then `id`, the order the
/// listings use; a `NULL` `sort_order` sorts last). A module the user already
/// completed is accepted again whatever the state of the previous ones
/// (idempotence).
///
/// The access check, the order check and the upsert run as a single statement
/// (MAIR-420), so the check cannot go stale before the write. The enrolment row
/// is locked `FOR UPDATE`: an unenrolment in flight
/// (`UnsubUserFormationQueryView`) either waits for this statement to commit,
/// then deletes the progress it wrote, or commits first, in which case this
/// statement finds no enrolment and writes nothing. Read the result with
/// `fetch_one` into a [`CompleteModuleRow`]. The database-side trigger
/// (`fn_update_user_course_progress`) rolls the completion up into the parent
/// `user_courses` row, which it updates: that is why the lock is exclusive. With
/// `FOR SHARE`, two completions of the same user both held the shared lock,
/// then each waited for the other one in the trigger (deadlock); with
/// `FOR UPDATE` they queue.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompleteModuleQueryView {
    params: Vec<QueryParam>,
}

impl CompleteModuleQueryView {
    pub fn new(user_id: u64, formation_id: u64, module_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(formation_id as i32),
                QueryParam::I32(module_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn module_id(&self) -> u64 {
        self.params[2].as_i32() as u64
    }
}

impl ApiRequestDto for CompleteModuleQueryView {
    fn query_sql(&self) -> &'static str {
        // A data-modifying CTE must sit at the top level, hence the `WITH`
        // around the usual `to_jsonb(t)` select.
        "WITH enrolment AS ( \
            SELECT 1 FROM user_courses WHERE user_id = $1 AND course_id = $2 FOR UPDATE \
         ), state AS ( \
            SELECT \
                EXISTS(SELECT 1 FROM enrolment) AS enrolled, \
                EXISTS( \
                    SELECT 1 FROM course_modules WHERE id = $3 AND course_id = $2 \
                ) AS module_in_formation, \
                EXISTS( \
                    SELECT 1 FROM user_modules \
                    WHERE user_id = $1 AND module_id = $3 AND is_completed \
                ) AS already_completed, \
                EXISTS( \
                    SELECT 1 \
                    FROM course_modules target \
                    JOIN course_modules prev ON prev.course_id = target.course_id \
                        AND (COALESCE(prev.sort_order, 2147483647), prev.id) \
                            < (COALESCE(target.sort_order, 2147483647), target.id) \
                    LEFT JOIN user_modules um \
                        ON um.module_id = prev.id AND um.user_id = $1 AND um.is_completed \
                    WHERE target.id = $3 AND um.module_id IS NULL \
                ) AS previous_pending \
         ), decision AS ( \
            SELECT enrolled, module_in_formation, \
                enrolled AND module_in_formation \
                    AND (already_completed OR NOT previous_pending) AS completed \
            FROM state \
         ), upserted AS ( \
            INSERT INTO user_modules (user_id, module_id, is_completed, completed_at) \
            SELECT $1, $3, TRUE, CURRENT_TIMESTAMP FROM decision WHERE completed \
            ON CONFLICT (user_id, module_id) \
            DO UPDATE SET is_completed = TRUE, completed_at = CURRENT_TIMESTAMP \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT enrolled, module_in_formation, completed FROM decision \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of a [`CompleteModuleQueryView`]. Nothing was written unless
/// [`Self::completed`] is `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompleteModuleRow {
    enrolled: bool,
    module_in_formation: bool,
    completed: bool,
}

impl CompleteModuleRow {
    /// The user is enrolled in the course of the path.
    pub fn enrolled(&self) -> bool {
        self.enrolled
    }

    /// The module exists and belongs to the course of the path.
    pub fn module_in_formation(&self) -> bool {
        self.module_in_formation
    }

    /// The module is now completed. `false` when the user is not enrolled, the
    /// module is not in the course, or a previous module of the course is not
    /// completed yet.
    pub fn completed(&self) -> bool {
        self.completed
    }
}
