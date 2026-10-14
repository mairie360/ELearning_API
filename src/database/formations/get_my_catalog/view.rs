use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Lists the formations a user is enrolled in (`user_courses`) with, nested in the same row, their
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
        // `file_url` (the S3 object key) is deliberately not selected: a viewable URL is only
        // issued by the single-attachment route.
        "SELECT to_jsonb(t) FROM ( \
            SELECT c.id, c.title AS name, c.description, uc.status::text AS status, \
                COALESCE(( \
                    SELECT jsonb_agg( \
                        jsonb_build_object( \
                            'id', cm.id, \
                            'name', cm.title, \
                            'description', cm.content, \
                            'completed', COALESCE(um.is_completed, FALSE), \
                            'files', COALESCE(( \
                                SELECT jsonb_agg( \
                                    jsonb_build_object( \
                                        'id', ca.id, \
                                        'file_name', ca.file_name, \
                                        'file_type', ca.file_type::text, \
                                        'file_size_bytes', ca.file_size_bytes \
                                    ) ORDER BY ca.id \
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
pub struct CatalogFileRow {
    id: i32,
    file_name: String,
    file_type: String,
    file_size_bytes: Option<i64>,
}

impl CatalogFileRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }

    pub fn file_size_bytes(&self) -> Option<i64> {
        self.file_size_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CatalogModuleRow {
    id: i32,
    name: String,
    description: Option<String>,
    completed: bool,
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

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn files(&self) -> &[CatalogFileRow] {
        &self.files
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CatalogFormationRow {
    id: i32,
    name: String,
    description: Option<String>,
    status: String,
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

    pub fn modules(&self) -> &[CatalogModuleRow] {
        &self.modules
    }
}
