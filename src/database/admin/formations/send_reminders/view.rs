use crate::database::admin::stats::get_stats::view::STATS_CACHE_NAMESPACE;
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Records a reminder (`course_reminders`) for agents of a mandatory formation (MAIR-1150), in one
/// statement with its checks: the formation must exist and be mandatory, and only non-archived
/// agents enrolled in it who have not completed it are reminded.
///
/// `user_ids` names the agents to remind (the others are ignored); `None` reminds every late agent
/// of the formation, those whose deadline is past. `sent_by` is the administrator. Read with
/// `fetch_one` into a [`SendRemindersRow`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SendRemindersQueryView {
    params: Vec<QueryParam>,
}

impl SendRemindersQueryView {
    pub fn new(formation_id: u64, user_ids: Option<&[u64]>, sent_by: u64) -> Self {
        // A JSON array of the ids, or JSON `null` for "every late agent".
        let targets = match user_ids {
            Some(ids) => serde_json::Value::from(
                ids.iter()
                    .map(|id| serde_json::Value::from(id_to_sql(*id)))
                    .collect::<Vec<_>>(),
            ),
            None => serde_json::Value::Null,
        };
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::Text(targets.to_string()),
                QueryParam::I32(id_to_sql(sent_by)),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for SendRemindersQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH course AS ( \
            SELECT id, is_mandatory, deadline FROM courses WHERE id = $1 \
         ), targets AS ( \
            SELECT uc.user_id \
            FROM user_courses uc \
            JOIN course c ON c.id = uc.course_id \
            JOIN users u ON u.id = uc.user_id AND u.is_archived IS NOT TRUE \
            WHERE c.is_mandatory AND uc.status <> 'completed' \
                AND CASE \
                    WHEN jsonb_typeof($2::jsonb) = 'array' THEN uc.user_id IN ( \
                        SELECT value::int FROM jsonb_array_elements_text($2::jsonb)) \
                    ELSE c.deadline < CURRENT_DATE \
                END \
         ), inserted AS ( \
            INSERT INTO course_reminders (course_id, user_id, sent_by) \
            SELECT $1, user_id, $3 FROM targets \
            RETURNING id \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT EXISTS(SELECT 1 FROM course) AS course_exists, \
                COALESCE((SELECT is_mandatory FROM course), FALSE) AS mandatory, \
                (SELECT count(*) FROM inserted) AS sent \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        &[STATS_CACHE_NAMESPACE]
    }
}

/// Outcome of a [`SendRemindersQueryView`]: nothing is recorded unless the formation exists and
/// is mandatory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SendRemindersRow {
    course_exists: bool,
    mandatory: bool,
    sent: i64,
}

impl SendRemindersRow {
    pub fn course_exists(&self) -> bool {
        self.course_exists
    }

    pub fn mandatory(&self) -> bool {
        self.mandatory
    }

    /// Number of reminders recorded.
    pub fn sent(&self) -> i64 {
        self.sent
    }
}
