use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// A file a `PATCH /admin/formations/{formation_id}/` keeps, with the type the request gives it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeptFile {
    pub id: i32,
    pub file_type: String,
}

/// Checks the ids a course update keeps (MAIR-817), in one statement: the module ids that are not
/// modules of the course, the file ids that are not files of one of its modules, and the kept
/// files whose type would change to or from a stored file (`video`, `pdf`, `document`, `audio`:
/// an S3 object the API cannot upload nor drop). Read with `fetch_one` into a
/// [`CourseContentCheckRow`]; every list is empty when the request is consistent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckCourseContentQueryView {
    params: Vec<QueryParam>,
}

impl CheckCourseContentQueryView {
    pub fn new(formation_id: u64, module_ids: &[u64], files: &[KeptFile]) -> Self {
        let module_ids: Vec<i32> = module_ids.iter().map(|id| id_to_sql(*id)).collect();
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::Text(serde_json::to_string(&module_ids).unwrap_or_default()),
                QueryParam::Text(serde_json::to_string(files).unwrap_or_default()),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for CheckCourseContentQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH modules AS ( \
            SELECT value::int AS id FROM jsonb_array_elements_text($2::jsonb) \
         ), files AS ( \
            SELECT id, file_type FROM jsonb_to_recordset($3::jsonb) AS f(id int, file_type text) \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT \
                COALESCE((SELECT jsonb_agg(m.id) FROM modules m WHERE NOT EXISTS ( \
                    SELECT 1 FROM course_modules cm WHERE cm.id = m.id AND cm.course_id = $1 \
                )), '[]'::jsonb) AS unknown_modules, \
                COALESCE((SELECT jsonb_agg(f.id) FROM files f WHERE NOT EXISTS ( \
                    SELECT 1 FROM course_attachments ca \
                    JOIN course_modules cm ON cm.id = ca.module_id \
                    WHERE ca.id = f.id AND cm.course_id = $1 \
                )), '[]'::jsonb) AS unknown_files, \
                COALESCE((SELECT jsonb_agg(f.id) FROM files f \
                    JOIN course_attachments ca ON ca.id = f.id \
                    WHERE ca.file_type::text <> f.file_type \
                        AND (ca.file_type::text IN ('video', 'pdf', 'document', 'audio') \
                            OR f.file_type IN ('video', 'pdf', 'document', 'audio')) \
                ), '[]'::jsonb) AS retyped_files \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of a [`CheckCourseContentQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CourseContentCheckRow {
    unknown_modules: Vec<i32>,
    unknown_files: Vec<i32>,
    retyped_files: Vec<i32>,
}

impl CourseContentCheckRow {
    /// Module ids of the request that are not modules of the course.
    pub fn unknown_modules(&self) -> &[i32] {
        &self.unknown_modules
    }

    /// File ids of the request that are not files of the course.
    pub fn unknown_files(&self) -> &[i32] {
        &self.unknown_files
    }

    /// Kept files whose type would change to or from a stored file.
    pub fn retyped_files(&self) -> &[i32] {
        &self.retyped_files
    }
}
