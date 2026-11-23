use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::send_reminders::view::{
    SendRemindersQueryView, SendRemindersRow,
};
use crate::endpoints::v1::admin::formations::formation_id::reminders::view::{
    SendRemindersResultView, SendRemindersView,
};
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationIdParams;
use crate::endpoints::validation::ValidatedJson;
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum SendRemindersError {
    NotFound,
    NotMandatory,
    DatabaseError,
}

impl std::fmt::Display for SendRemindersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendRemindersError::NotFound => write!(f, "The formation was not found."),
            SendRemindersError::NotMandatory => {
                write!(f, "Only a mandatory formation can be reminded.")
            }
            SendRemindersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for SendRemindersError {
    fn status_code(&self) -> StatusCode {
        match self {
            SendRemindersError::NotFound => StatusCode::NOT_FOUND,
            SendRemindersError::NotMandatory => StatusCode::CONFLICT,
            SendRemindersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_send_reminders(
    state: web::Data<AppState>,
    admin_id: u64,
    formation_id: u64,
    body: SendRemindersView,
) -> Result<SendRemindersResultView, SendRemindersError> {
    // Checks and inserts in one statement (MAIR-420).
    let view = SendRemindersQueryView::new(formation_id, body.user_ids(), admin_id);
    let outcome: SendRemindersRow =
        state
            .get_smart_db()
            .fetch_one(&view)
            .await
            .map_err(log_error(
                "trigger_send_reminders",
                SendRemindersError::DatabaseError,
            ))?;
    if !outcome.course_exists() {
        return Err(SendRemindersError::NotFound);
    }
    if !outcome.mandatory() {
        return Err(SendRemindersError::NotMandatory);
    }
    Ok(SendRemindersResultView {
        sent: u64::try_from(outcome.sent()).unwrap_or_default(),
    })
}

#[utoipa::path(
    post,
    params(AdminFormationIdParams),
    path = "/reminders/",
    summary = "Remind agents of a mandatory formation",
    description = "Records a reminder for agents of a mandatory formation: each one then sees it \
                   as `last_reminder_at` in `GET /api/v1/formations/catalog/`, and the indicators \
                   of `GET /api/v1/admin/stats/` show it in `late_agents`.\n\n\
                   With `user_ids`, the named agents are reminded; without it (`{}`), every late \
                   agent of the formation is (deadline past, not completed). Only non-archived \
                   agents enrolled in the formation who have not completed it are reminded: the \
                   other ids are ignored, so `sent` may be lower than the number of ids. No \
                   message is sent by this API: the reminder is a record the fronts show.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    request_body(
        content = SendRemindersView,
        description = "The agents to remind, or `{}` for every late agent.",
        example = json!({ "user_ids": [42] })
    ),
    responses(
        (
            status = 200,
            description = "Reminders recorded.",
            body = SendRemindersResultView,
            example = json!({ "sent": 1 })
        ),
        (
            status = 400,
            description = "Malformed body, an id outside 1 to 2147483647, more than 1000 ids, or a path segment that is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `user_ids[0]`: must be between 1 and 2147483647")
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
            status = 409,
            description = "The formation is not mandatory. Nothing is recorded.",
            body = String,
            content_type = "text/plain",
            example = json!("Only a mandatory formation can be reminded.")
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
#[post("/reminders/")]
pub async fn send_reminders(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<AdminFormationIdParams>,
    body: ValidatedJson<SendRemindersView>,
) -> Result<impl Responder, SendRemindersError> {
    let result = trigger_send_reminders(
        state,
        auth_user.id,
        params.formation_id(),
        body.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(result))
}
