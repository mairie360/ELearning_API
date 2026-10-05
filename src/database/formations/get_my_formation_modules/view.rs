use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Lists a course's modules along with whether the given user has completed
/// each one (`LEFT JOIN` on `user_modules`, defaulting to `false`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMyFormationModulesQueryView {
    params: Vec<QueryParam>,
}

impl GetMyFormationModulesQueryView {
    pub fn new(formation_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::I32(id_to_sql(user_id)),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for GetMyFormationModulesQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT cm.id, cm.title AS name, cm.content AS description, \
                COALESCE(um.is_completed, FALSE) AS completed \
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
pub struct FormationModuleRow {
    id: i32,
    name: String,
    description: Option<String>,
    completed: bool,
}

impl FormationModuleRow {
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
}
