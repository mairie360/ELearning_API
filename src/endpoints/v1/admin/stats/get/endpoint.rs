use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;

use crate::database::admin::stats::get_stats::view::{
    FormationStatsRow, GetStatsQueryView, LateAgentRow, ServiceStatsRow, StatsRow, STATS_CACHE,
};
use crate::endpoints::v1::admin::stats::get::view::{
    FormationStats, GetStatsResultView, LateAgent, ServiceStats,
};
use crate::endpoints::v1::formations::catalog::view::CourseSource;
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum GetStatsError {
    DatabaseError,
}

impl std::fmt::Display for GetStatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetStatsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetStatsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetStatsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

fn count(value: i64) -> u64 {
    u64::try_from(value).unwrap_or_default()
}

/// `part / whole` in percent, one decimal; 0 when `whole` is 0.
pub fn rate(part: i64, whole: i64) -> f64 {
    if whole <= 0 {
        return 0.0;
    }
    // Counts of rows stay far below 2^52: the conversion is exact.
    #[allow(clippy::cast_precision_loss)]
    let ratio = part as f64 * 100.0 / whole as f64;
    (ratio * 10.0).round() / 10.0
}

fn map_service(row: &ServiceStatsRow) -> ServiceStats {
    ServiceStats {
        service_id: row.service_id().map(id_from_sql),
        service_name: row.service_name().map(str::to_string),
        agents: count(row.agents()),
        participants: count(row.participants()),
        participation_rate: rate(row.participants(), row.agents()),
    }
}

fn map_formation(row: &FormationStatsRow) -> FormationStats {
    FormationStats {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
        source: CourseSource::from_db(row.source()),
        is_mandatory: row.is_mandatory(),
        enrolled: count(row.enrolled()),
        completed: count(row.completed()),
        average_progress: row.average_progress(),
    }
}

fn map_late_agent(row: &LateAgentRow) -> LateAgent {
    LateAgent {
        user_id: id_from_sql(row.user_id()),
        first_name: row.first_name().to_string(),
        last_name: row.last_name().to_string(),
        formation_id: id_from_sql(row.formation_id()),
        formation_name: row.formation_name().to_string(),
        deadline: row.deadline(),
        progress: row.progress(),
        last_reminder_at: row.last_reminder_at().map(|dt| dt.and_utc()),
    }
}

/// The indicators, read from the Redis cache ([`STATS_CACHE`], 60 s, shared by every admin) or
/// computed and stored there. Without Redis they are computed on every call.
///
/// # Errors
///
/// Returns the database error of the statement; nothing is cached then.
pub async fn load_stats(db: &SmartDatabase) -> Result<StatsRow, ApiLibError> {
    db.cached(&STATS_CACHE.shared(&()), async || {
        db.fetch_one(&GetStatsQueryView::new()).await
    })
    .await
}

async fn trigger_get_stats(
    state: web::Data<AppState>,
) -> Result<GetStatsResultView, GetStatsError> {
    let row = load_stats(state.get_smart_db())
        .await
        .map_err(log_error("trigger_get_stats", GetStatsError::DatabaseError))?;

    Ok(GetStatsResultView {
        internal_courses: count(row.internal_courses()),
        platform_courses: count(row.platform_courses()),
        mandatory_internal_courses: count(row.mandatory_internal_courses()),
        agents: count(row.agents()),
        participants: count(row.participants()),
        participation_rate: rate(row.participants(), row.agents()),
        average_completion: row.average_completion(),
        late_mandatory: count(row.late_mandatory()),
        by_service: row.by_service().iter().map(map_service).collect(),
        by_formation: row.by_formation().iter().map(map_formation).collect(),
        late_agents: row.late_agents().iter().map(map_late_agent).collect(),
    })
}

#[utoipa::path(
    get,
    path = "",
    summary = "Indicators of the e-learning",
    description = "Returns the indicators of the e-learning administration, computed over the \
                   non-archived accounts in one statement: formations by origin, \
                   participation (overall and per service of the mairie), average progress of \
                   the enrolments, enrolments per formation and the agents late on a mandatory \
                   formation (deadline past, not completed) with their latest reminder.\n\n\
                   The progress of an enrolment is the share of the modules of the formation the \
                   agent completed (100 once completed, 0 for a formation without modules).\n\n\
                   **Cache:** the indicators are cached in Redis for 60 seconds, one entry \
                   shared by every admin. Creating, replacing or deleting a formation and \
                   sending reminders invalidate it, so an admin reads those changes \
                   immediately; the learners' activity (enrolments, completed modules) shows up \
                   within those 60 seconds. Without Redis they are computed on every call.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Indicators of the instance.",
            body = GetStatsResultView,
            example = json!({
                "internal_courses": 18,
                "platform_courses": 6,
                "mandatory_internal_courses": 4,
                "agents": 120,
                "participants": 90,
                "participation_rate": 75.0,
                "average_completion": 62.4,
                "late_mandatory": 7,
                "by_service": [
                    { "service_id": 3, "service_name": "Civil status", "agents": 12, "participants": 9, "participation_rate": 75.0 },
                    { "service_id": null, "service_name": null, "agents": 4, "participants": 1, "participation_rate": 25.0 }
                ],
                "by_formation": [
                    { "id": 4, "name": "GDPR for local government staff", "source": "internal", "is_mandatory": true, "enrolled": 40, "completed": 25, "average_progress": 71.3 }
                ],
                "late_agents": [
                    { "user_id": 42, "first_name": "Jean", "last_name": "Martin", "formation_id": 4, "formation_name": "GDPR for local government staff", "deadline": "2026-06-30", "progress": 50.0, "last_reminder_at": null }
                ]
            })
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
            description = "The caller is authenticated but is not an admin (checked by `AdminMiddleware` before the handler runs).",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden: User is not an admin.")
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
    tag = "Admin - Indicators",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_stats(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
) -> Result<impl Responder, GetStatsError> {
    let stats = trigger_get_stats(state).await?;
    Ok(HttpResponse::Ok().json(stats))
}

#[cfg(test)]
mod tests {
    use super::rate;

    #[test]
    fn rates_are_percentages_with_one_decimal() {
        assert!((rate(0, 0) - 0.0).abs() < f64::EPSILON);
        assert!((rate(9, 12) - 75.0).abs() < f64::EPSILON);
        assert!((rate(1, 3) - 33.3).abs() < 1e-9);
        assert!((rate(2, 3) - 66.7).abs() < 1e-9);
    }
}
