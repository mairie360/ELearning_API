use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Deletes a user's per-module progress on a course: `user_modules` references
/// `course_modules`, not `user_courses`, so nothing cascades from an
/// unenrolment and a later re-registration would otherwise resurrect the old
/// completions. Progress on other courses is untouched.
///
/// Run with `execute`, in the transaction of the unenrolment, after
/// `UnsubUserFormationQueryView` (see its documentation for the ordering).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PurgeUserFormationProgressQueryView {
    params: Vec<QueryParam>,
}

impl PurgeUserFormationProgressQueryView {
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

impl ApiRequestDto for PurgeUserFormationProgressQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM user_modules um \
         USING course_modules cm \
         WHERE um.module_id = cm.id AND cm.course_id = $2 AND um.user_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
