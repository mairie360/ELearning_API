use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists a single course's modules. Mirrors
/// `admin::formations::get_formations`'s module shape but scoped to one
/// `course_id`, for `GET /admin/formations/{formation_id}`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetFormationModulesQueryView {
    params: Vec<QueryParam>,
}

impl GetFormationModulesQueryView {
    pub fn new(formation_id: u64, details: bool) -> Self {
        Self {
            params: vec![
                QueryParam::I32(formation_id as i32),
                QueryParam::Bool(details),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn details(&self) -> bool {
        self.params[1].as_bool()
    }
}

impl ApiRequestDto for GetFormationModulesQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT cm.id, cm.title AS name, cm.content AS description, \
                CASE WHEN $2 THEN ( \
                    SELECT json_agg(json_build_object( \
                        'id', ca.id, \
                        'file_name', ca.file_name, \
                        'file_type', ca.file_type::text \
                    ) ORDER BY ca.id) \
                    FROM course_attachments ca WHERE ca.module_id = cm.id \
                ) END AS content \
            FROM course_modules cm \
            WHERE cm.course_id = $1 \
            ORDER BY cm.sort_order, cm.id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdminModuleContentRow {
    id: i32,
    file_name: String,
    file_type: String,
}

impl AdminModuleContentRow {
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
pub struct AdminFormationModuleRow {
    id: i32,
    name: String,
    description: Option<String>,
    content: Option<Vec<AdminModuleContentRow>>,
}

impl AdminFormationModuleRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn content(&self) -> Option<&Vec<AdminModuleContentRow>> {
        self.content.as_ref()
    }
}
