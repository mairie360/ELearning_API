use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::rate_formation::view::{RateFormationQueryView, RateFormationRow};
use crate::endpoints::v1::formations::formation_id::rating::view::{
    RateFormationView, RatingResultView,
};
use crate::endpoints::v1::formations::formation_id::FormationIdParams;
use crate::endpoints::validation::ValidatedJson;
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum RateFormationError {
    Forbidden,
    NotCompleted,
    DatabaseError,
}

impl std::fmt::Display for RateFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateFormationError::Forbidden => write!(f, "You are not enrolled in this formation."),
            RateFormationError::NotCompleted => {
                write!(f, "Complete the formation before rating it.")
            }
            RateFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for RateFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            RateFormationError::Forbidden => StatusCode::FORBIDDEN,
            RateFormationError::NotCompleted => StatusCode::CONFLICT,
            RateFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_rate_formation(
    state: web::Data<AppState>,
    user_id: u64,
    formation_id: u64,
    rating: u8,
) -> Result<RatingResultView, RateFormationError> {
    // Checks and upsert in one statement (MAIR-420).
    let view = RateFormationQueryView::new(user_id, formation_id, rating);
    let outcome: RateFormationRow =
        state
            .get_smart_db()
            .fetch_one(&view)
            .await
            .map_err(log_error(
                "trigger_rate_formation",
                RateFormationError::DatabaseError,
            ))?;

    if !outcome.enrolled() {
        return Err(RateFormationError::Forbidden);
    }
    if !outcome.completed() {
        return Err(RateFormationError::NotCompleted);
    }
    Ok(RatingResultView {
        rating_average: outcome.rating_average().unwrap_or(f64::from(rating)),
        rating_count: u64::try_from(outcome.rating_count()).unwrap_or(1),
        my_rating: outcome
            .my_rating()
            .and_then(|recorded| u8::try_from(recorded).ok())
            .unwrap_or(rating),
    })
}

#[utoipa::path(
    post,
    params(FormationIdParams),
    path = "",
    summary = "Rate one of your formations",
    description = "Records the rating (1 to 5) the caller gives a formation, or replaces the one \
                   they gave before. Only a caller enrolled in the formation (`403` otherwise, \
                   admins included, and for an unknown formation) who completed it (`409` \
                   otherwise) may rate it.\n\n\
                   Answers the ratings of the formation once this one is recorded; \
                   `GET /api/v1/formations/catalog/` shows them too (`rating_average`, \
                   `rating_count`, `my_rating`).",
    request_body(
        content = RateFormationView,
        description = "The rating, from 1 to 5.",
        example = json!({ "rating": 4 })
    ),
    responses(
        (
            status = 200,
            description = "Rating recorded.",
            body = RatingResultView,
            example = json!({ "rating_average": 4.3, "rating_count": 13, "my_rating": 4 })
        ),
        (
            status = 400,
            description = "Malformed body, `rating` outside 1 to 5, or a path segment that is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `rating`: must be between 1 and 5")
        ),
        (
            status = 401,
            description = "`Authorization` header missing, or JWT invalid or expired.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller is not enrolled in this formation, or the formation does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
        ),
        (
            status = 409,
            description = "The caller has not completed the formation yet. Nothing is recorded.",
            body = String,
            content_type = "text/plain",
            example = json!("Complete the formation before rating it.")
        ),
        (
            status = 429,
            description = "The caller exceeded their request quota (per user, `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST`). `Retry-After` gives the seconds to wait.",
            body = String,
            content_type = "text/plain",
            example = json!("Too many requests, retry in 1 s.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Formations",
)]
#[post("/rating/")]
pub async fn rate_formation(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<FormationIdParams>,
    body: ValidatedJson<RateFormationView>,
) -> Result<impl Responder, RateFormationError> {
    let rating = body.into_inner().rating();
    let result = trigger_rate_formation(state, auth_user.id, params.formation_id(), rating).await?;
    Ok(HttpResponse::Ok().json(result))
}
