use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Registers a user to a course (`user_courses`, defaulting to
/// `not_started`) in a single statement: the existence checks of the user and
/// of the course and the insert run on the same snapshot, so there is no
/// window between "both exist" and "insert" for a concurrent request.
/// Idempotent: registering the same pair twice is a no-op.
///
/// Read the result with `fetch_one` into a [`RegisterUserToFormationRow`]. A
/// user or course deleted by a concurrent transaction after the snapshot can
/// still trip the foreign keys (`DbError::ForeignKeyViolation`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterUserToFormationQueryView {
    params: Vec<QueryParam>,
}

impl RegisterUserToFormationQueryView {
    pub fn new(user_id: u64, formation_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(formation_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn formation_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for RegisterUserToFormationQueryView {
    fn query_sql(&self) -> &'static str {
        // A data-modifying CTE must sit at the top level, hence the `WITH`
        // around the usual `to_jsonb(t)` select.
        "WITH found AS ( \
            SELECT \
                EXISTS(SELECT 1 FROM users WHERE id = $1) AS user_exists, \
                EXISTS(SELECT 1 FROM courses WHERE id = $2) AS course_exists \
         ), inserted AS ( \
            INSERT INTO user_courses (user_id, course_id) \
            SELECT $1, $2 FROM found WHERE user_exists AND course_exists \
            ON CONFLICT (user_id, course_id) DO NOTHING \
         ) \
         SELECT to_jsonb(t) FROM (SELECT user_exists, course_exists FROM found) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of a [`RegisterUserToFormationQueryView`]: the registration was
/// made (or already existed) only when both flags are `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegisterUserToFormationRow {
    user_exists: bool,
    course_exists: bool,
}

impl RegisterUserToFormationRow {
    /// The user of the request exists.
    pub fn user_exists(&self) -> bool {
        self.user_exists
    }

    /// The course of the path exists.
    pub fn course_exists(&self) -> bool {
        self.course_exists
    }
}
