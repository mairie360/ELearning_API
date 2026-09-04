use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists the files attached to a module, scoped to the parent course so a
/// mismatched `(formation_id, module_id)` pair simply returns no rows.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetModuleAttachmentsQueryView {
    params: Vec<QueryParam>,
}

impl GetModuleAttachmentsQueryView {
    pub fn new(formation_id: u64, module_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(module_id as i32),
                QueryParam::I32(formation_id as i32),
            ],
        }
    }

    pub fn module_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for GetModuleAttachmentsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT ca.id, ca.file_name, ca.file_type::text AS file_type, ca.file_url, \
                ca.file_size_bytes \
            FROM course_attachments ca \
            JOIN course_modules cm ON cm.id = ca.module_id \
            WHERE ca.module_id = $1 AND cm.course_id = $2 \
            ORDER BY ca.id \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleAttachmentRow {
    id: i32,
    file_name: String,
    file_type: String,
    file_url: String,
    file_size_bytes: Option<i64>,
}

impl ModuleAttachmentRow {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }

    pub fn file_url(&self) -> &str {
        &self.file_url
    }

    pub fn file_size_bytes(&self) -> Option<i64> {
        self.file_size_bytes
    }
}
