use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Which rows of a course a [`DeleteCourseContentQueryView`] deletes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CourseContent {
    /// The files of the course's modules.
    Files,
    /// The modules of the course (their files and the progress on them go with them).
    Modules,
}

/// Deletes the files (or the modules) of a course whose ids are not in `kept`: the rows a course
/// update no longer lists (MAIR-817). A course update deletes the files first, saves the modules
/// and files of the request, then deletes the modules, so a kept file moved to another module is
/// not taken away with its former module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteCourseContentQueryView {
    content: CourseContent,
    params: Vec<QueryParam>,
}

impl DeleteCourseContentQueryView {
    pub fn new(content: CourseContent, formation_id: u64, kept: &[u64]) -> Self {
        let kept: Vec<i32> = kept.iter().map(|id| id_to_sql(*id)).collect();
        Self {
            content,
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::Text(serde_json::to_string(&kept).unwrap_or_default()),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for DeleteCourseContentQueryView {
    fn query_sql(&self) -> &'static str {
        match self.content {
            CourseContent::Files => {
                "DELETE FROM course_attachments ca USING course_modules cm \
                 WHERE ca.module_id = cm.id AND cm.course_id = $1 \
                    AND ca.id NOT IN (SELECT value::int FROM jsonb_array_elements_text($2::jsonb))"
            }
            CourseContent::Modules => {
                "DELETE FROM course_modules WHERE course_id = $1 \
                    AND id NOT IN (SELECT value::int FROM jsonb_array_elements_text($2::jsonb))"
            }
        }
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
