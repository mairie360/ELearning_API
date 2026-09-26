use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Authorization check shared by every route under
/// `/v1/formations/{formation_id}/{module_id}`: tells in one round trip whether
/// the caller is enrolled in the course (`user_courses`) and whether the module
/// of the path belongs to that course. Never cached: an unenrolment must take
/// effect on the next request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckModuleAccessQueryView {
    params: Vec<QueryParam>,
}

impl CheckModuleAccessQueryView {
    pub fn new(user_id: u64, formation_id: u64, module_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(formation_id as i32),
                QueryParam::I32(module_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn module_id(&self) -> u64 {
        self.params[2].as_i32() as u64
    }
}

impl ApiRequestDto for CheckModuleAccessQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT \
                EXISTS(SELECT 1 FROM user_courses WHERE user_id = $1 AND course_id = $2) \
                    AS enrolled, \
                EXISTS(SELECT 1 FROM course_modules WHERE id = $3 AND course_id = $2) \
                    AS module_in_formation \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleAccessRow {
    enrolled: bool,
    module_in_formation: bool,
}

impl ModuleAccessRow {
    /// The user has a `user_courses` row for the course of the path.
    pub fn enrolled(&self) -> bool {
        self.enrolled
    }

    /// The module of the path exists and belongs to the course of the path.
    pub fn module_in_formation(&self) -> bool {
        self.module_in_formation
    }
}
