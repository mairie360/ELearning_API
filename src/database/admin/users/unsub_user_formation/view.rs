use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Unregisters a user from a course. Read the result with `fetch_one` into an
/// [`UnsubUserFormationRow`]: nothing is deleted when the user was not
/// registered to the course.
///
/// The user's per-module progress on the course is deleted afterwards by
/// [`PurgeUserFormationProgressQueryView`], **in the same transaction and in a
/// second statement** (MAIR-420): this `DELETE` waits for a completion in flight
/// (`CompleteModuleQueryView` locks the enrolment row `FOR UPDATE`), and only a
/// statement started after that wait sees the progress it wrote.
///
/// [`PurgeUserFormationProgressQueryView`]: crate::database::admin::users::purge_user_formation_progress::view::PurgeUserFormationProgressQueryView
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
        "WITH unregistered AS ( \
            DELETE FROM user_courses WHERE user_id = $1 AND course_id = $2 \
            RETURNING user_id \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT EXISTS(SELECT 1 FROM unregistered) AS unregistered \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of an [`UnsubUserFormationQueryView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnsubUserFormationRow {
    unregistered: bool,
}

impl UnsubUserFormationRow {
    /// `false` when the user was not registered to the course (or the course
    /// does not exist): nothing was deleted.
    pub fn unregistered(&self) -> bool {
        self.unregistered
    }
}
