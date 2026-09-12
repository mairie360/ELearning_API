use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Fetches a single attachment, scoped to its parent module *and* course so a
/// mismatched `(formation_id, module_id, attachment_id)` triple simply returns
/// no row (mapped to a 404 by the endpoint).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetAttachmentQueryView {
    params: Vec<QueryParam>,
}

impl GetAttachmentQueryView {
    pub fn new(formation_id: u64, module_id: u64, attachment_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(attachment_id as i32),
                QueryParam::I32(module_id as i32),
                QueryParam::I32(formation_id as i32),
            ],
        }
    }

    pub fn attachment_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn module_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[2].as_i32() as u64
    }
}

impl ApiRequestDto for GetAttachmentQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT ca.id, ca.file_name, ca.file_type::text AS file_type, ca.file_url \
            FROM course_attachments ca \
            JOIN course_modules cm ON cm.id = ca.module_id \
            WHERE ca.id = $1 AND ca.module_id = $2 AND cm.course_id = $3 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttachmentRow {
    id: i32,
    file_name: String,
    file_type: String,
    /// The S3 object key (the `course_attachments.file_url` column).
    file_url: String,
}

impl AttachmentRow {
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
}
