use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Records the rating (1 to 5) a user gives a formation (upsert on the `(user_id, course_id)`
/// primary key of `course_ratings`), only when the user is enrolled in it and completed it.
///
/// The checks and the upsert run as one statement (MAIR-420): the enrolment row is locked
/// `FOR SHARE`, so an unenrolment waits for the rating to commit. The answer carries the ratings of
/// the formation after the write: the statement's snapshot does not see its own upsert, so they
/// are computed from the other users' ratings plus the one just written. Read it with `fetch_one`
/// into a [`RateFormationRow`]; nothing is written unless [`RateFormationRow::completed`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateFormationQueryView {
    params: Vec<QueryParam>,
}

impl RateFormationQueryView {
    pub fn new(user_id: u64, formation_id: u64, rating: u8) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(formation_id)),
                QueryParam::I32(i32::from(rating)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn formation_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for RateFormationQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH enrolment AS ( \
            SELECT status FROM user_courses WHERE user_id = $1 AND course_id = $2 FOR SHARE \
         ), decision AS ( \
            SELECT EXISTS(SELECT 1 FROM enrolment) AS enrolled, \
                EXISTS(SELECT 1 FROM enrolment WHERE status = 'completed') AS completed \
         ), upserted AS ( \
            INSERT INTO course_ratings (user_id, course_id, rating) \
            SELECT $1, $2, $3::smallint FROM decision WHERE completed \
            ON CONFLICT (user_id, course_id) \
            DO UPDATE SET rating = EXCLUDED.rating, updated_at = now() \
            RETURNING rating \
         ), ratings AS ( \
            SELECT rating FROM course_ratings WHERE course_id = $2 AND user_id <> $1 \
            UNION ALL SELECT rating FROM upserted \
         ) \
         SELECT to_jsonb(t) FROM ( \
            SELECT d.enrolled, d.completed, \
                (SELECT round(avg(rating)::numeric, 1) FROM ratings) AS rating_average, \
                (SELECT count(*) FROM ratings) AS rating_count, \
                (SELECT rating FROM upserted) AS my_rating \
            FROM decision d \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Outcome of a [`RateFormationQueryView`].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RateFormationRow {
    enrolled: bool,
    completed: bool,
    rating_average: Option<f64>,
    rating_count: i64,
    my_rating: Option<i16>,
}

impl RateFormationRow {
    /// The user is enrolled in the formation.
    pub fn enrolled(&self) -> bool {
        self.enrolled
    }

    /// The user completed the formation: the rating was recorded.
    pub fn completed(&self) -> bool {
        self.completed
    }

    /// Average rating of the formation after the write, one decimal.
    pub fn rating_average(&self) -> Option<f64> {
        self.rating_average
    }

    /// Number of ratings of the formation after the write.
    pub fn rating_count(&self) -> i64 {
        self.rating_count
    }

    /// The rating recorded, `None` when nothing was written.
    pub fn my_rating(&self) -> Option<i16> {
        self.my_rating
    }
}
