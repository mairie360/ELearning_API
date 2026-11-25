use chrono::{NaiveDate, NaiveDateTime};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::ReadCache;

/// Redis namespace of the cached indicators.
pub const STATS_CACHE_NAMESPACE: &str = "elearning_stats";
/// Seconds the indicators stay cached: the statement aggregates every enrolment and completed
/// module (about 80 ms of database time on 30 000 enrolments), so it runs at most once per
/// window and per pod instead of once per admin page view.
pub const STATS_CACHE_TTL_SECONDS: u64 = 60;
/// One entry shared by every admin (the indicators do not depend on who asks). The admin writes
/// on the formations invalidate it; the learners' progress shows up within the TTL.
pub const STATS_CACHE: ReadCache = ReadCache::new(STATS_CACHE_NAMESPACE, STATS_CACHE_TTL_SECONDS);

/// Most overdue enrolments [`GetStatsQueryView`] lists in `late_agents`.
pub const MAX_LATE_AGENTS: i32 = 500;

/// The indicators of the e-learning administration (MAIR-1150), in one statement over the
/// non-archived accounts: course counts by origin, participation (overall and per service of the
/// mairie), average progress, per-formation enrolments and the agents late on a mandatory
/// formation.
///
/// The progress of an enrolment is the share of the modules of the formation the agent completed
/// (100 once the enrolment is `completed`, 0 for a formation without modules). An enrolment is late
/// when its formation is mandatory, its deadline is past and it is not completed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetStatsQueryView {
    params: Vec<QueryParam>,
}

impl Default for GetStatsQueryView {
    fn default() -> Self {
        Self::new()
    }
}

impl GetStatsQueryView {
    pub fn new() -> Self {
        Self {
            params: vec![QueryParam::I32(MAX_LATE_AGENTS)],
        }
    }
}

impl ApiRequestDto for GetStatsQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH agents AS ( \
            SELECT id, first_name, last_name, service_id FROM users \
            WHERE is_archived IS NOT TRUE \
         ), module_counts AS ( \
            SELECT course_id, count(*) AS modules FROM course_modules GROUP BY course_id \
         ), done AS ( \
            SELECT um.user_id, cm.course_id, count(*) AS done \
            FROM user_modules um JOIN course_modules cm ON cm.id = um.module_id \
            WHERE um.is_completed GROUP BY um.user_id, cm.course_id \
         ), enrolments AS ( \
            SELECT uc.user_id, uc.course_id, uc.status, c.title, c.source, c.is_mandatory, \
                c.deadline, \
                CASE \
                    WHEN uc.status = 'completed' THEN 100.0 \
                    WHEN COALESCE(mc.modules, 0) = 0 THEN 0.0 \
                    ELSE LEAST(100.0, 100.0 * COALESCE(d.done, 0) / mc.modules) \
                END AS progress \
            FROM user_courses uc \
            JOIN agents a ON a.id = uc.user_id \
            JOIN courses c ON c.id = uc.course_id \
            LEFT JOIN module_counts mc ON mc.course_id = uc.course_id \
            LEFT JOIN done d ON d.user_id = uc.user_id AND d.course_id = uc.course_id \
         ), participants AS ( \
            SELECT DISTINCT user_id FROM enrolments \
         ), late AS ( \
            SELECT e.* FROM enrolments e \
            WHERE e.is_mandatory AND e.deadline < CURRENT_DATE AND e.status <> 'completed' \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT \
                (SELECT count(*) FROM courses WHERE source = 'internal') AS internal_courses, \
                (SELECT count(*) FROM courses WHERE source = 'platform') AS platform_courses, \
                (SELECT count(*) FROM courses WHERE source = 'internal' AND is_mandatory) \
                    AS mandatory_internal_courses, \
                (SELECT count(*) FROM agents) AS agents, \
                (SELECT count(*) FROM participants) AS participants, \
                COALESCE((SELECT round(avg(progress), 1) FROM enrolments), 0) \
                    AS average_completion, \
                (SELECT count(*) FROM late) AS late_mandatory, \
                COALESCE(( \
                    SELECT jsonb_agg(jsonb_build_object( \
                        'service_id', s.service_id, \
                        'service_name', s.service_name, \
                        'agents', s.agents, \
                        'participants', s.participants \
                    ) ORDER BY s.service_name NULLS LAST, s.service_id) \
                    FROM ( \
                        SELECT a.service_id, sv.name AS service_name, count(*) AS agents, \
                            count(p.user_id) AS participants \
                        FROM agents a \
                        LEFT JOIN services sv ON sv.id = a.service_id \
                        LEFT JOIN participants p ON p.user_id = a.id \
                        GROUP BY a.service_id, sv.name \
                    ) s \
                ), '[]'::jsonb) AS by_service, \
                COALESCE(( \
                    SELECT jsonb_agg(jsonb_build_object( \
                        'id', f.id, \
                        'name', f.title, \
                        'source', f.source, \
                        'is_mandatory', f.is_mandatory, \
                        'enrolled', f.enrolled, \
                        'completed', f.completed, \
                        'average_progress', f.average_progress \
                    ) ORDER BY f.id) \
                    FROM ( \
                        SELECT c.id, c.title, c.source, c.is_mandatory, \
                            count(e.user_id) AS enrolled, \
                            count(e.user_id) FILTER (WHERE e.status = 'completed') AS completed, \
                            COALESCE(round(avg(e.progress), 1), 0) AS average_progress \
                        FROM courses c \
                        LEFT JOIN enrolments e ON e.course_id = c.id \
                        GROUP BY c.id \
                    ) f \
                ), '[]'::jsonb) AS by_formation, \
                COALESCE(( \
                    SELECT jsonb_agg(jsonb_build_object( \
                        'user_id', l.user_id, \
                        'first_name', l.first_name, \
                        'last_name', l.last_name, \
                        'formation_id', l.course_id, \
                        'formation_name', l.title, \
                        'deadline', l.deadline, \
                        'progress', l.progress, \
                        'last_reminder_at', l.last_reminder_at \
                    ) ORDER BY l.deadline, l.course_id, l.user_id) \
                    FROM ( \
                        SELECT late.user_id, a.first_name, a.last_name, late.course_id, \
                            late.title, late.deadline, round(late.progress, 1) AS progress, \
                            (SELECT max(cr.sent_at) FROM course_reminders cr \
                             WHERE cr.user_id = late.user_id AND cr.course_id = late.course_id) \
                                AS last_reminder_at \
                        FROM late JOIN agents a ON a.id = late.user_id \
                        ORDER BY late.deadline, late.course_id, late.user_id \
                        LIMIT $1 \
                    ) l \
                ), '[]'::jsonb) AS late_agents \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Participation of the agents of one service, in a [`StatsRow`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServiceStatsRow {
    service_id: Option<i32>,
    service_name: Option<String>,
    agents: i64,
    participants: i64,
}

impl ServiceStatsRow {
    /// `None` for the agents without a service.
    pub fn service_id(&self) -> Option<i32> {
        self.service_id
    }

    pub fn service_name(&self) -> Option<&str> {
        self.service_name.as_deref()
    }

    pub fn agents(&self) -> i64 {
        self.agents
    }

    pub fn participants(&self) -> i64 {
        self.participants
    }
}

/// Enrolments of one formation, in a [`StatsRow`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormationStatsRow {
    id: i32,
    name: String,
    source: String,
    is_mandatory: bool,
    enrolled: i64,
    completed: i64,
    average_progress: f64,
}

impl FormationStatsRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn is_mandatory(&self) -> bool {
        self.is_mandatory
    }

    pub fn enrolled(&self) -> i64 {
        self.enrolled
    }

    pub fn completed(&self) -> i64 {
        self.completed
    }

    pub fn average_progress(&self) -> f64 {
        self.average_progress
    }
}

/// An agent late on a mandatory formation, in a [`StatsRow`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LateAgentRow {
    user_id: i32,
    first_name: String,
    last_name: String,
    formation_id: i32,
    formation_name: String,
    deadline: NaiveDate,
    progress: f64,
    last_reminder_at: Option<NaiveDateTime>,
}

impl LateAgentRow {
    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn formation_id(&self) -> i32 {
        self.formation_id
    }

    pub fn formation_name(&self) -> &str {
        &self.formation_name
    }

    pub fn deadline(&self) -> NaiveDate {
        self.deadline
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn last_reminder_at(&self) -> Option<NaiveDateTime> {
        self.last_reminder_at
    }
}

/// Result of a [`GetStatsQueryView`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StatsRow {
    internal_courses: i64,
    platform_courses: i64,
    mandatory_internal_courses: i64,
    agents: i64,
    participants: i64,
    average_completion: f64,
    late_mandatory: i64,
    by_service: Vec<ServiceStatsRow>,
    by_formation: Vec<FormationStatsRow>,
    late_agents: Vec<LateAgentRow>,
}

impl StatsRow {
    pub fn internal_courses(&self) -> i64 {
        self.internal_courses
    }

    pub fn platform_courses(&self) -> i64 {
        self.platform_courses
    }

    pub fn mandatory_internal_courses(&self) -> i64 {
        self.mandatory_internal_courses
    }

    /// Non-archived accounts.
    pub fn agents(&self) -> i64 {
        self.agents
    }

    /// Non-archived accounts enrolled in at least one formation.
    pub fn participants(&self) -> i64 {
        self.participants
    }

    /// Average progress over the enrolments, in percent, one decimal.
    pub fn average_completion(&self) -> f64 {
        self.average_completion
    }

    /// Number of late enrolments on mandatory formations (all of them, not only the listed ones).
    pub fn late_mandatory(&self) -> i64 {
        self.late_mandatory
    }

    pub fn by_service(&self) -> &[ServiceStatsRow] {
        &self.by_service
    }

    pub fn by_formation(&self) -> &[FormationStatsRow] {
        &self.by_formation
    }

    /// The [`MAX_LATE_AGENTS`] most overdue enrolments.
    pub fn late_agents(&self) -> &[LateAgentRow] {
        &self.late_agents
    }
}
