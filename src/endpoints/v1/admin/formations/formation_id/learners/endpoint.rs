use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::get_formation_learners::view::{
    FormationLearnersRow, GetFormationLearnersQueryView, LearnerRow,
};
use crate::endpoints::v1::admin::formations::formation_id::learners::view::{
    FormationLearner, GetFormationLearnersResultView,
};
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationIdParams;
use crate::endpoints::v1::admin::users::ProgressStatus;
use crate::endpoints::v1::admin::AdminPageQuery;
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum GetFormationLearnersError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetFormationLearnersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetFormationLearnersError::NotFound => write!(f, "The formation was not found."),
            GetFormationLearnersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetFormationLearnersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetFormationLearnersError::NotFound => StatusCode::NOT_FOUND,
            GetFormationLearnersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

fn map_learner(row: &LearnerRow) -> FormationLearner {
    FormationLearner {
        user_id: id_from_sql(row.user_id()),
        first_name: row.first_name().to_string(),
        last_name: row.last_name().to_string(),
        status: ProgressStatus::from(row.status().to_string()),
        progress: u8::try_from(row.progress().clamp(0, 100)).unwrap_or_default(),
        started_at: row.started_at().map(|dt| dt.and_utc()),
        completed_at: row.completed_at().map(|dt| dt.and_utc()),
        last_activity_at: row.last_activity_at().map(|dt| dt.and_utc()),
        last_reminder_at: row.last_reminder_at().map(|dt| dt.and_utc()),
    }
}

async fn trigger_get_formation_learners(
    state: web::Data<AppState>,
    formation_id: u64,
    page: &AdminPageQuery,
) -> Result<GetFormationLearnersResultView, GetFormationLearnersError> {
    let view = GetFormationLearnersQueryView::new(formation_id, page.limit(), page.offset());
    let row: FormationLearnersRow =
        state
            .get_smart_db()
            .fetch_one(&view)
            .await
            .map_err(log_error(
                "trigger_get_formation_learners",
                GetFormationLearnersError::DatabaseError,
            ))?;
    if !row.course_exists() {
        return Err(GetFormationLearnersError::NotFound);
    }
    Ok(GetFormationLearnersResultView {
        formation_id,
        total: u64::try_from(row.total()).unwrap_or_default(),
        limit: page.limit(),
        offset: page.offset(),
        learners: row.learners().iter().map(map_learner).collect(),
    })
}

#[utoipa::path(
    get,
    params(AdminFormationIdParams, AdminPageQuery),
    path = "/learners/",
    summary = "List the learners of a formation",
    description = "Returns one page of the agents enrolled in the formation, with their progress, \
                   ordered by last name, first name and id. Archived accounts are left out, of \
                   the page and of `total`.\n\n\
                   `progress` is the share of the modules of the formation the agent completed \
                   (rounded down, 100 once the formation is completed). `last_reminder_at` is \
                   the latest reminder recorded by `POST \
                   /api/v1/admin/formations/{formation_id}/reminders/`.\n\n\
                   Paginated: `limit` (default 50, capped to 200) and `offset` (default 0); \
                   `total` counts every learner, so the last page is the one reaching it.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "One page of the learners of the formation, possibly empty.",
            body = GetFormationLearnersResultView,
            example = json!({
                "formation_id": 4,
                "total": 2,
                "limit": 50,
                "offset": 0,
                "learners": [
                    {
                        "user_id": 51,
                        "first_name": "Amina",
                        "last_name": "Bensaïd",
                        "status": "Completed",
                        "progress": 100,
                        "started_at": "2026-09-01T09:00:00Z",
                        "completed_at": "2026-09-10T16:45:00Z",
                        "last_activity_at": "2026-09-10T16:45:00Z",
                        "last_reminder_at": null
                    },
                    {
                        "user_id": 42,
                        "first_name": "Jean",
                        "last_name": "Dupont",
                        "status": "InProgress",
                        "progress": 50,
                        "started_at": "2026-09-02T08:30:00Z",
                        "completed_at": null,
                        "last_activity_at": "2026-09-03T14:25:00Z",
                        "last_reminder_at": "2026-10-01T08:00:00Z"
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "`limit` or `offset` is not a non-negative integer, or a path segment is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Query deserialize error: invalid digit found in string")
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
            status = 404,
            description = "The formation does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("The formation was not found.")
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
    tag = "Admin - Formations",
    security(
        ("jwt" = [])
    )
)]
#[get("/learners/")]
pub async fn get_formation_learners(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminFormationIdParams>,
    page: web::Query<AdminPageQuery>,
) -> Result<impl Responder, GetFormationLearnersError> {
    let result = trigger_get_formation_learners(state, params.formation_id(), &page).await?;
    Ok(HttpResponse::Ok().json(result))
}
