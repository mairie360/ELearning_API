use chrono::{NaiveDate, NaiveDateTime};
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Lists the formations a user is enrolled in (`user_courses`), or every formation when the user is an
/// administrator (`is_admin()`: an administrator manages formations they never followed; those come out
/// `not_started` with no activity dates), with, nested in the same row, their
/// modules (and whether the user completed each one) and the files of each module. One statement
/// for the whole catalogue: the nested lists are correlated subqueries aggregated into JSON, so the
/// number of queries does not grow with the number of formations or modules.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMyCatalogQueryView {
    params: Vec<QueryParam>,
}

impl GetMyCatalogQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id))],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetMyCatalogQueryView {
    fn query_sql(&self) -> &'static str {
        // `file_url` (the S3 object key) is only selected for a `link`, whose value is the URL the
        // agent opens: a viewable URL of a stored file is only issued by the single-attachment
        // route. The number of learners per formation is counted once for the caller's
        // formations (`learners`), not by a correlated subquery per formation.
        "WITH mine AS ( \
            SELECT course_id FROM user_courses WHERE user_id = $1 \
            UNION \
            SELECT id FROM courses WHERE (SELECT is_admin($1)) \
         ), learners AS ( \
            SELECT course_id, count(*) AS learners FROM user_courses \
            WHERE course_id IN (SELECT course_id FROM mine) GROUP BY course_id \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT c.id, c.title AS name, c.description, COALESCE(uc.status::text, 'not_started') AS status, \
                c.category, c.level, c.is_mandatory, c.deadline, c.source, \
                COALESCE( \
                    NULLIF(btrim(c.instructor_label), ''), \
                    (SELECT iu.first_name || ' ' || iu.last_name FROM users iu \
                     WHERE iu.id = c.instructor_user_id) \
                ) AS instructor, \
                uc.started_at, uc.completed_at, uc.last_activity_at, \
                (SELECT max(cr.sent_at) FROM course_reminders cr \
                 WHERE cr.user_id = $1 AND cr.course_id = c.id) AS last_reminder_at, \
                (SELECT round(avg(r.rating)::numeric, 1) FROM course_ratings r \
                 WHERE r.course_id = c.id) AS rating_average, \
                (SELECT count(*) FROM course_ratings r WHERE r.course_id = c.id) AS rating_count, \
                (SELECT r.rating FROM course_ratings r \
                 WHERE r.course_id = c.id AND r.user_id = $1) AS my_rating, \
                COALESCE(l.learners, 0) AS learners, \
                COALESCE(( \
                    SELECT jsonb_agg( \
                        jsonb_build_object( \
                            'id', cm.id, \
                            'name', cm.title, \
                            'description', cm.content, \
                            'sort_order', cm.sort_order, \
                            'completed', COALESCE(um.is_completed, FALSE), \
                            'duration_seconds', COALESCE(( \
                                SELECT sum(ca.duration_seconds) FROM course_attachments ca \
                                WHERE ca.module_id = cm.id \
                            ), 0), \
                            'files', COALESCE(( \
                                SELECT jsonb_agg( \
                                    jsonb_build_object( \
                                        'id', ca.id, \
                                        'file_name', COALESCE(ca.file_name, ca.title), \
                                        'file_type', ca.file_type::text, \
                                        'file_size_bytes', ca.file_size_bytes, \
                                        'title', ca.title, \
                                        'description', ca.description, \
                                        'duration_seconds', ca.duration_seconds, \
                                        'is_required', ca.is_required, \
                                        'sort_order', ca.sort_order, \
                                        'url', CASE WHEN ca.file_type = 'link' THEN ca.file_url END \
                                    ) ORDER BY ca.sort_order, ca.id \
                                ) \
                                FROM course_attachments ca \
                                WHERE ca.module_id = cm.id \
                            ), '[]'::jsonb) \
                        ) ORDER BY cm.sort_order, cm.id \
                    ) \
                    FROM course_modules cm \
                    LEFT JOIN user_modules um ON um.module_id = cm.id AND um.user_id = $1 \
                    WHERE cm.course_id = c.id \
                ), '[]'::jsonb) AS modules \
            FROM mine m \
            JOIN courses c ON c.id = m.course_id \
            LEFT JOIN user_courses uc ON uc.course_id = c.id AND uc.user_id = $1 \
            LEFT JOIN learners l ON l.course_id = c.id \
            ORDER BY c.id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CatalogFileRow {
    id: i32,
    file_name: String,
    file_type: String,
    file_size_bytes: Option<i64>,
    title: String,
    description: Option<String>,
    duration_seconds: Option<i32>,
    is_required: bool,
    sort_order: i32,
    url: Option<String>,
}

impl CatalogFileRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// The `attachment_type` as stored: `video`, `pdf`, `document`, `link`, `quiz`, `audio` or
    /// `other`.
    pub fn file_type(&self) -> &str {
        &self.file_type
    }

    pub fn file_size_bytes(&self) -> Option<i64> {
        self.file_size_bytes
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn duration_seconds(&self) -> Option<i32> {
        self.duration_seconds
    }

    pub fn is_required(&self) -> bool {
        self.is_required
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }

    /// The URL of a `link`, `None` for every other type.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CatalogModuleRow {
    id: i32,
    name: String,
    description: Option<String>,
    sort_order: Option<i32>,
    completed: bool,
    duration_seconds: i64,
    files: Vec<CatalogFileRow>,
}

impl CatalogModuleRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn sort_order(&self) -> Option<i32> {
        self.sort_order
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    /// Sum of the durations of the files of the module, 0 when none has one.
    pub fn duration_seconds(&self) -> i64 {
        self.duration_seconds
    }

    pub fn files(&self) -> &[CatalogFileRow] {
        &self.files
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CatalogFormationRow {
    id: i32,
    name: String,
    description: Option<String>,
    status: String,
    category: Option<String>,
    level: Option<String>,
    is_mandatory: bool,
    deadline: Option<NaiveDate>,
    source: String,
    instructor: Option<String>,
    started_at: Option<NaiveDateTime>,
    completed_at: Option<NaiveDateTime>,
    last_activity_at: Option<NaiveDateTime>,
    last_reminder_at: Option<NaiveDateTime>,
    rating_average: Option<f64>,
    rating_count: i64,
    my_rating: Option<i16>,
    learners: i64,
    modules: Vec<CatalogModuleRow>,
}

impl CatalogFormationRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn level(&self) -> Option<&str> {
        self.level.as_deref()
    }

    pub fn is_mandatory(&self) -> bool {
        self.is_mandatory
    }

    pub fn deadline(&self) -> Option<NaiveDate> {
        self.deadline
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// The instructor label, else the name of the instructor account.
    pub fn instructor(&self) -> Option<&str> {
        self.instructor.as_deref()
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

    /// Latest reminder sent to the caller about this formation.
    pub fn last_reminder_at(&self) -> Option<NaiveDateTime> {
        self.last_reminder_at
    }

    /// Average rating, rounded to one decimal; `None` without rating.
    pub fn rating_average(&self) -> Option<f64> {
        self.rating_average
    }

    pub fn rating_count(&self) -> i64 {
        self.rating_count
    }

    pub fn my_rating(&self) -> Option<i16> {
        self.my_rating
    }

    /// Number of users enrolled in the formation.
    pub fn learners(&self) -> i64 {
        self.learners
    }

    pub fn modules(&self) -> &[CatalogModuleRow] {
        &self.modules
    }
}
