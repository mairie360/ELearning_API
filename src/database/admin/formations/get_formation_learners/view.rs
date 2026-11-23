use chrono::NaiveDateTime;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// One page of the learners of a formation (MAIR-817): the non-archived agents enrolled in it
/// (`user_courses`), ordered by last name, first name and id, with their progress, in one
/// statement. The row also says whether the formation exists, how many learners it has in all and
/// how many modules it counts. The reminder date and the completed modules are correlated
/// subqueries of the page only, so the cost does not grow with the learners outside the page.
///
/// Read with `fetch_one` into a [`FormationLearnersRow`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetFormationLearnersQueryView {
    params: Vec<QueryParam>,
}

impl GetFormationLearnersQueryView {
    /// The caller bounds `limit`.
    pub fn new(formation_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetFormationLearnersQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH course AS ( \
            SELECT id FROM courses WHERE id = $1 \
         ), module_count AS ( \
            SELECT count(*) AS modules FROM course_modules WHERE course_id = $1 \
         ), enrolled AS ( \
            SELECT u.id AS user_id, u.first_name, u.last_name, uc.status::text AS status, \
                uc.started_at, uc.completed_at, uc.last_activity_at \
            FROM user_courses uc \
            JOIN users u ON u.id = uc.user_id \
            WHERE uc.course_id = $1 AND u.is_archived IS NOT TRUE \
         ), page AS ( \
            SELECT e.*, \
                (SELECT max(cr.sent_at) FROM course_reminders cr \
                 WHERE cr.course_id = $1 AND cr.user_id = e.user_id) AS last_reminder_at, \
                (SELECT count(*) FROM user_modules um \
                 JOIN course_modules cm ON cm.id = um.module_id \
                 WHERE cm.course_id = $1 AND um.user_id = e.user_id AND um.is_completed) \
                    AS completed_modules \
            FROM enrolled e \
            ORDER BY e.last_name, e.first_name, e.user_id \
            LIMIT $2 OFFSET $3 \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT EXISTS(SELECT 1 FROM course) AS course_exists, \
                (SELECT count(*) FROM enrolled) AS total, \
                COALESCE(( \
                    SELECT jsonb_agg( \
                        jsonb_build_object( \
                            'user_id', p.user_id, \
                            'first_name', p.first_name, \
                            'last_name', p.last_name, \
                            'status', p.status, \
                            'progress', CASE \
                                WHEN p.status = 'completed' THEN 100 \
                                WHEN mc.modules = 0 THEN 0 \
                                ELSE LEAST(100, 100 * p.completed_modules / mc.modules) \
                            END, \
                            'started_at', p.started_at, \
                            'completed_at', p.completed_at, \
                            'last_activity_at', p.last_activity_at, \
                            'last_reminder_at', p.last_reminder_at \
                        ) ORDER BY p.last_name, p.first_name, p.user_id \
                    ) \
                    FROM page p CROSS JOIN module_count mc \
                ), '[]'::jsonb) AS learners \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// A learner of a formation, in a [`FormationLearnersRow`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LearnerRow {
    user_id: i32,
    first_name: String,
    last_name: String,
    status: String,
    progress: i64,
    started_at: Option<NaiveDateTime>,
    completed_at: Option<NaiveDateTime>,
    last_activity_at: Option<NaiveDateTime>,
    last_reminder_at: Option<NaiveDateTime>,
}

impl LearnerRow {
    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    /// The `progress_status` as stored: `not_started`, `in_progress` or `completed`.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Completed modules over the modules of the formation, from 0 to 100 (100 once the
    /// formation is completed, 0 for a formation without module).
    pub fn progress(&self) -> i64 {
        self.progress
    }

    pub fn started_at(&self) -> Option<NaiveDateTime> {
        self.started_at
    }

    pub fn completed_at(&self) -> Option<NaiveDateTime> {
        self.completed_at
    }

    pub fn last_activity_at(&self) -> Option<NaiveDateTime> {
        self.last_activity_at
    }

    pub fn last_reminder_at(&self) -> Option<NaiveDateTime> {
        self.last_reminder_at
    }
}

/// Result of a [`GetFormationLearnersQueryView`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormationLearnersRow {
    course_exists: bool,
    total: i64,
    learners: Vec<LearnerRow>,
}

impl FormationLearnersRow {
    pub fn course_exists(&self) -> bool {
        self.course_exists
    }

    /// Non-archived learners of the formation, all pages together.
    pub fn total(&self) -> i64 {
        self.total
    }

    /// The learners of the page.
    pub fn learners(&self) -> &[LearnerRow] {
        &self.learners
    }
}
