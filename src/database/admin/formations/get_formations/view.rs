use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists every course. When `details` is `true`, each course's modules (and
/// each module's attachments) are aggregated as nested JSON in the same
/// round-trip instead of running one query per course.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetFormationsQueryView {
    params: Vec<QueryParam>,
}

impl GetFormationsQueryView {
    pub fn new(details: bool) -> Self {
        Self {
            params: vec![QueryParam::Bool(details)],
        }
    }

    pub fn details(&self) -> bool {
        self.params[0].as_bool()
    }
}

impl ApiRequestDto for GetFormationsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT c.id, c.title AS name, c.description, \
                CASE WHEN $1 THEN ( \
                    SELECT json_agg(json_build_object( \
                        'id', cm.id, \
                        'name', cm.title, \
                        'description', cm.content, \
                        'content', ( \
                            SELECT json_agg(json_build_object( \
                                'id', ca.id, \
                                'file_name', ca.file_name, \
                                'file_type', ca.file_type::text \
                            ) ORDER BY ca.id) \
                            FROM course_attachments ca WHERE ca.module_id = cm.id \
                        ) \
                    ) ORDER BY cm.sort_order, cm.id) \
                    FROM course_modules cm WHERE cm.course_id = c.id \
                ) END AS modules \
            FROM courses c \
            ORDER BY c.id \
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdminFormationRow {
    id: i32,
    name: String,
    description: Option<String>,
    modules: Option<Vec<AdminFormationModuleRow>>,
}

impl AdminFormationRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn modules(&self) -> Option<&Vec<AdminFormationModuleRow>> {
        self.modules.as_ref()
    }
}
