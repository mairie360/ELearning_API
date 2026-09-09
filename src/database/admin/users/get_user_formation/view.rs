use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists a single course's modules with one specific user's completion
/// state. `content` is always an array (never `null`): it's just empty when
/// `details` is `false`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUserFormationQueryView {
    params: Vec<QueryParam>,
}

impl GetUserFormationQueryView {
    pub fn new(formation_id: u64, user_id: u64, details: bool) -> Self {
        Self {
            params: vec![
                QueryParam::I32(formation_id as i32),
                QueryParam::I32(user_id as i32),
                QueryParam::Bool(details),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn user_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn details(&self) -> bool {
        self.params[2].as_bool()
    }
}

impl ApiRequestDto for GetUserFormationQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT cm.id, cm.title AS name, cm.content AS description, \
                CASE WHEN $3 THEN COALESCE(( \
                    SELECT json_agg(json_build_object( \
                        'id', ca.id, \
                        'file_name', ca.file_name, \
                        'file_type', ca.file_type::text \
                    ) ORDER BY ca.id) \
                    FROM course_attachments ca WHERE ca.module_id = cm.id \
                ), '[]'::json) ELSE '[]'::json END AS content, \
                COALESCE(um.is_completed, FALSE) AS is_completed, \
                um.completed_at \
            FROM course_modules cm \
            LEFT JOIN user_modules um ON um.module_id = cm.id AND um.user_id = $2 \
            WHERE cm.course_id = $1 \
            ORDER BY cm.sort_order, cm.id \
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
