use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// The fields of a module an administrator sets, bound as a JSON object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleFields {
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

/// Inserts a module in a course (`module_id` `None`) or updates one of its modules; `fetch_scalar`
/// reads the id of the module. An update of a module that is not in the course writes nothing and
/// returns no row (the caller checked the ids first, under the lock of the course row).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveModuleQueryView {
    params: Vec<QueryParam>,
}

impl SaveModuleQueryView {
    pub fn new(formation_id: u64, module_id: Option<u64>, fields: &ModuleFields) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::OptionI32(module_id.map(id_to_sql)),
                QueryParam::Text(serde_json::to_string(fields).unwrap_or_default()),
            ],
        }
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for SaveModuleQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH input AS ( \
            SELECT $3::jsonb AS j \
         ), updated AS ( \
            UPDATE course_modules SET title = i.j->>'name', content = i.j->>'description', \
                sort_order = (i.j->>'sort_order')::int \
            FROM input i \
            WHERE id = $2 AND course_id = $1 \
            RETURNING id \
         ), inserted AS ( \
            INSERT INTO course_modules (course_id, title, content, sort_order) \
            SELECT $1, j->>'name', j->>'description', (j->>'sort_order')::int \
            FROM input WHERE $2::int IS NULL \
            RETURNING id \
         ) \
         SELECT id FROM updated UNION ALL SELECT id FROM inserted"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
