use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Lists the courses a user is registered to (`user_courses`), joined with the
/// course's own title/description.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMyFormationsQueryView {
    params: Vec<QueryParam>,
}

impl GetMyFormationsQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(user_id as i32)],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for GetMyFormationsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT c.id, c.title AS name, c.description, uc.status::text AS status \
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
pub struct FormationSummaryRow {
    id: i32,
    name: String,
    description: Option<String>,
    status: String,
}

impl FormationSummaryRow {
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
}
