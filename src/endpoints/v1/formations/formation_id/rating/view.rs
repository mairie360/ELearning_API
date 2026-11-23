use crate::endpoints::validation::{Validate, ValidationError};

/// Rating the caller gives a formation they completed.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RateFormationView {
    /// From 1 (poor) to 5 (excellent).
    #[schema(example = 4, minimum = 1, maximum = 5)]
    rating: u8,
}

impl RateFormationView {
    pub fn new(rating: u8) -> Self {
        Self { rating }
    }

    pub fn rating(&self) -> u8 {
        self.rating
    }
}

impl Validate for RateFormationView {
    fn validate(&self) -> Result<(), ValidationError> {
        if !(1..=5).contains(&self.rating) {
            return Err(ValidationError::new("rating", "must be between 1 and 5"));
        }
        Ok(())
    }
}

/// Ratings of the formation once the caller's is recorded.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RatingResultView {
    /// Average rating of the formation, one decimal.
    #[schema(example = 4.3)]
    pub rating_average: f64,
    /// Number of ratings of the formation, the caller's included.
    #[schema(example = 13)]
    pub rating_count: u64,
    /// The rating the caller just gave.
    #[schema(example = 4)]
    pub my_rating: u8,
}
