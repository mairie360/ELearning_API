use chrono::NaiveDateTime;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Training time of a user over a calendar year (MAIR-1144), in one statement: the yearly goal of
/// the instance (`elearning_settings`), the time spent, the formations completed that year and,
/// per formation the user is enrolled in, its status and the time spent on it that year.
///
/// The time spent on a formation is the sum of the `duration_seconds` of the contents of the
/// modules the user completed during the year (`user_modules.completed_at`); a content without a
/// duration counts 0.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMySummaryQueryView {
    params: Vec<QueryParam>,
}

impl GetMySummaryQueryView {
    pub fn new(user_id: u64, year: i32) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id)), QueryParam::I32(year)],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn year(&self) -> i32 {
        self.params[1].as_i32()
    }
}

impl ApiRequestDto for GetMySummaryQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH bounds AS ( \
            SELECT make_date($2, 1, 1)::timestamp AS year_start, \
                make_date($2 + 1, 1, 1)::timestamp AS year_end \
         ), enrolments AS ( \
            SELECT uc.course_id, c.title, uc.status::text AS status, uc.completed_at \
            FROM user_courses uc JOIN courses c ON c.id = uc.course_id \
            WHERE uc.user_id = $1 \
         ), spent AS ( \
            SELECT cm.course_id, \
                COALESCE(sum(ca.duration_seconds), 0)::bigint AS seconds \
            FROM user_modules um \
            JOIN course_modules cm ON cm.id = um.module_id \
            JOIN course_attachments ca ON ca.module_id = cm.id \
            CROSS JOIN bounds b \
            WHERE um.user_id = $1 AND um.is_completed \
                AND um.completed_at >= b.year_start AND um.completed_at < b.year_end \
            GROUP BY cm.course_id \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT \
                (SELECT annual_goal_minutes FROM elearning_settings WHERE id = 1) \
                    AS annual_goal_minutes, \
                COALESCE((SELECT sum(s.seconds) FROM spent s \
                    JOIN enrolments e ON e.course_id = s.course_id), 0)::bigint \
                    AS time_spent_seconds, \
                (SELECT count(*) FROM enrolments e CROSS JOIN bounds b \
                    WHERE e.status = 'completed' \
                        AND e.completed_at >= b.year_start AND e.completed_at < b.year_end) \
                    AS completed_count, \
                COALESCE(( \
                    SELECT jsonb_agg(jsonb_build_object( \
                        'id', e.course_id, \
                        'name', e.title, \
                        'status', e.status, \
                        'time_spent_seconds', COALESCE(s.seconds, 0), \
                        'completed_at', e.completed_at \
                    ) ORDER BY e.course_id) \
                    FROM enrolments e LEFT JOIN spent s ON s.course_id = e.course_id \
                ), '[]'::jsonb) AS formations \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// A formation of the user in a [`SummaryRow`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryFormationRow {
    id: i32,
    name: String,
    status: String,
    time_spent_seconds: i64,
    completed_at: Option<NaiveDateTime>,
}

impl SummaryFormationRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    /// Seconds spent on the formation during the year.
    pub fn time_spent_seconds(&self) -> i64 {
        self.time_spent_seconds
    }

    pub fn completed_at(&self) -> Option<NaiveDateTime> {
        self.completed_at
    }
}

/// Result of a [`GetMySummaryQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryRow {
    annual_goal_minutes: Option<i32>,
    time_spent_seconds: i64,
    completed_count: i64,
    formations: Vec<SummaryFormationRow>,
}

impl SummaryRow {
    /// The yearly goal of the instance, `None` if its settings row is missing.
    pub fn annual_goal_minutes(&self) -> Option<i32> {
        self.annual_goal_minutes
    }

    pub fn time_spent_seconds(&self) -> i64 {
        self.time_spent_seconds
    }

    /// Formations completed during the year.
    pub fn completed_count(&self) -> i64 {
        self.completed_count
    }

    pub fn formations(&self) -> &[SummaryFormationRow] {
        &self.formations
    }
}
