use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// The fields of a file (content) an administrator sets, bound as a JSON object. `url` is only
/// stored for a `link`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileFields {
    pub title: String,
    pub description: Option<String>,
    pub file_type: String,
    pub duration_seconds: Option<u32>,
    pub is_required: bool,
    pub sort_order: i32,
    pub url: Option<String>,
}

/// Inserts a file in a module (`file_id` `None`) or updates a file, moving it to `module_id`;
/// `fetch_scalar` reads its id.
///
/// `file_url` holds the URL of a `link`. A kept stored file (video, PDF...) keeps its object key:
/// only its metadata change. A `link` turned into a `quiz` or `other` loses its URL. The name of
/// a content without a stored file follows its title.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveFileQueryView {
    params: Vec<QueryParam>,
}

impl SaveFileQueryView {
    pub fn new(module_id: u64, file_id: Option<u64>, fields: &FileFields) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(module_id)),
                QueryParam::OptionI32(file_id.map(id_to_sql)),
                QueryParam::Text(serde_json::to_string(fields).unwrap_or_default()),
            ],
        }
    }

    pub fn module_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for SaveFileQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH input AS ( \
            SELECT $3::jsonb AS j \
         ), updated AS ( \
            UPDATE course_attachments ca SET module_id = $1, \
                title = i.j->>'title', description = i.j->>'description', \
                file_type = (i.j->>'file_type')::attachment_type, \
                duration_seconds = (i.j->>'duration_seconds')::int, \
                is_required = (i.j->>'is_required')::boolean, \
                sort_order = (i.j->>'sort_order')::int, \
                file_url = CASE \
                    WHEN i.j->>'file_type' = 'link' THEN i.j->>'url' \
                    WHEN ca.file_type = 'link' THEN NULL \
                    ELSE ca.file_url \
                END, \
                file_name = CASE \
                    WHEN i.j->>'file_type' IN ('link', 'quiz', 'other') THEN i.j->>'title' \
                    ELSE ca.file_name \
                END \
            FROM input i \
            WHERE ca.id = $2 \
            RETURNING ca.id \
         ), inserted AS ( \
            INSERT INTO course_attachments (module_id, file_name, title, description, file_type, \
                file_url, duration_seconds, is_required, sort_order) \
            SELECT $1, j->>'title', j->>'title', j->>'description', \
                (j->>'file_type')::attachment_type, \
                CASE WHEN j->>'file_type' = 'link' THEN j->>'url' END, \
                (j->>'duration_seconds')::int, (j->>'is_required')::boolean, \
                (j->>'sort_order')::int \
            FROM input WHERE $2::int IS NULL \
            RETURNING id \
         ) \
         SELECT id FROM updated UNION ALL SELECT id FROM inserted"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
