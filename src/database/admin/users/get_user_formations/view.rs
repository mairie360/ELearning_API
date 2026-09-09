use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists the courses a given user is registered to, with their per-course
/// progress (`user_courses`). When `details` is `true`, each course's
/// modules are aggregated too, each carrying that user's own completion
/// state (`user_modules`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUserFormationsQueryView {
    params: Vec<QueryParam>,
}

impl GetUserFormationsQueryView {
    pub fn new(user_id: u64, details: bool) -> Self {
        Self {
            params: vec![QueryParam::I32(user_id as i32), QueryParam::Bool(details)],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn details(&self) -> bool {
        self.params[1].as_bool()
    }
}

impl ApiRequestDto for GetUserFormationsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT c.id, c.title AS name, c.description, \
                uc.started_at, uc.completed_at, uc.status::text AS progress_status, \
                CASE WHEN $2 THEN ( \
                    SELECT json_agg(json_build_object( \
                        'id', cm.id, \
                        'name', cm.title, \
                        'description', cm.content, \
                        'content', COALESCE(( \
                            SELECT json_agg(json_build_object( \
                                'id', ca.id, \
                                'file_name', ca.file_name, \
                                'file_type', ca.file_type::text \
                            ) ORDER BY ca.id) \
                            FROM course_attachments ca WHERE ca.module_id = cm.id \
                        ), '[]'::json), \
                        'is_completed', COALESCE(um.is_completed, FALSE), \
                        'completed_at', um.completed_at \
                    ) ORDER BY cm.sort_order, cm.id) \
                    FROM course_modules cm \
                    LEFT JOIN user_modules um ON um.module_id = cm.id AND um.user_id = $1 \
                    WHERE cm.course_id = c.id \
                ) END AS modules \
            FROM user_courses uc \
            JOIN courses c ON c.id = uc.course_id \
            WHERE uc.user_id = $1 \
            ORDER BY c.id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserModuleContentRow {
    id: i32,
    file_name: String,
    file_type: String,
}

impl UserModuleContentRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserFormationModuleRow {
    id: i32,
    name: String,
    description: Option<String>,
    content: Vec<UserModuleContentRow>,
    is_completed: bool,
    completed_at: Option<chrono::NaiveDateTime>,
}

impl UserFormationModuleRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn content(&self) -> &[UserModuleContentRow] {
        &self.content
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    pub fn completed_at(&self) -> Option<chrono::NaiveDateTime> {
        self.completed_at
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserFormationRow {
    id: i32,
    name: String,
    description: Option<String>,
    started_at: Option<chrono::NaiveDateTime>,
    completed_at: Option<chrono::NaiveDateTime>,
    progress_status: String,
    modules: Option<Vec<UserFormationModuleRow>>,
}

impl UserFormationRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn started_at(&self) -> Option<chrono::NaiveDateTime> {
        self.started_at
    }

    pub fn completed_at(&self) -> Option<chrono::NaiveDateTime> {
        self.completed_at
    }

    pub fn progress_status(&self) -> &str {
        &self.progress_status
    }

    pub fn modules(&self) -> Option<&Vec<UserFormationModuleRow>> {
        self.modules.as_ref()
    }
}
