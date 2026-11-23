use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_summary::view::{
    GetMySummaryQueryView, SummaryFormationRow, SummaryRow,
};
use crate::endpoints::v1::formations::get::view::Status;
use crate::endpoints::v1::formations::summary::view::{SummaryFormation, SummaryResultView};
use crate::endpoints::v1::formations::YearQuery;
use crate::error::log_error;

/// Goal used when the settings row of the instance is missing (the database default).
const DEFAULT_ANNUAL_GOAL_MINUTES: u32 = 600;

#[derive(Debug, Clone, PartialEq)]
pub enum GetSummaryError {
    InvalidYear,
    DatabaseError,
}

impl std::fmt::Display for GetSummaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetSummaryError::InvalidYear => {
                write!(f, "Invalid `year`: must be between 2000 and 2100")
            }
            GetSummaryError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetSummaryError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetSummaryError::InvalidYear => StatusCode::BAD_REQUEST,
            GetSummaryError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

fn non_negative(value: i64) -> u64 {
    u64::try_from(value).unwrap_or_default()
}

fn map_formation(row: &SummaryFormationRow) -> SummaryFormation {
    SummaryFormation {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
        status: Status::from(row.status().to_string()),
        time_spent_seconds: non_negative(row.time_spent_seconds()),
        completed_at: row.completed_at().map(|dt| dt.and_utc()),
    }
}

async fn trigger_get_summary(
    state: web::Data<AppState>,
    user_id: u64,
    year: i32,
) -> Result<SummaryResultView, GetSummaryError> {
    let view = GetMySummaryQueryView::new(user_id, year);
    let row: SummaryRow = state
        .get_smart_db()
        .fetch_one(&view)
        .await
        .map_err(log_error(
            "trigger_get_summary",
            GetSummaryError::DatabaseError,
        ))?;

    Ok(SummaryResultView {
        year,
        annual_goal_minutes: row
            .annual_goal_minutes()
            .and_then(|goal| u32::try_from(goal).ok())
            .unwrap_or(DEFAULT_ANNUAL_GOAL_MINUTES),
        time_spent_seconds: non_negative(row.time_spent_seconds()),
        completed_count: non_negative(row.completed_count()),
        formations: row.formations().iter().map(map_formation).collect(),
    })
}

#[utoipa::path(
    get,
    params(YearQuery),
    path = "",
    summary = "Your training time over a year",
    description = "Returns, for a calendar year (the current one by default), the yearly training \
                   goal of the instance, the time the caller spent training, the number of \
                   formations they completed and, for every formation they are enrolled in, its \
                   status and the time spent on it that year.\n\n\
                   The time spent is the sum of the durations (`duration_seconds`) of the contents \
                   of the modules the caller completed during the year: a content without a \
                   duration counts 0.",
    responses(
        (
            status = 200,
            description = "Training time of the caller over the year.",
            body = SummaryResultView,
            example = json!({
                "year": 2026,
                "annual_goal_minutes": 600,
                "time_spent_seconds": 5400,
                "completed_count": 1,
                "formations": [
                    { "id": 4, "name": "GDPR for local government staff", "status": "Completed", "time_spent_seconds": 3600, "completed_at": "2026-03-09T14:05:00Z" },
                    { "id": 7, "name": "Welcoming the public", "status": "InProgress", "time_spent_seconds": 1800, "completed_at": null }
                ]
            })
        ),
        (
            status = 400,
            description = "`year` is not an integer between 2000 and 2100.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `year`: must be between 2000 and 2100")
        ),
        (
            status = 401,
            description = "`Authorization` header missing, or JWT invalid or expired.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
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
#[get("/summary/")]
pub async fn get_my_summary(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    query: web::Query<YearQuery>,
) -> Result<impl Responder, GetSummaryError> {
    let year = query.year().ok_or(GetSummaryError::InvalidYear)?;
    let summary = trigger_get_summary(state, auth_user.id, year).await?;
    Ok(HttpResponse::Ok().json(summary))
}
