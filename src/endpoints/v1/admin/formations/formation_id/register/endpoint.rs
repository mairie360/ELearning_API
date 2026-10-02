use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;

use crate::database::admin::formations::register_user_to_formation::view::{
    RegisterUserToFormationQueryView, RegisterUserToFormationRow,
};
use crate::endpoints::v1::admin::formations::formation_id::register::view::RegisterUserView;
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationIdParams;
use crate::logging::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterUserToFormationError {
    BadRequest,
    DatabaseError,
    UnknownFormation,
    UnknownUser,
}

impl std::fmt::Display for RegisterUserToFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterUserToFormationError::BadRequest => {
                write!(f, "Bad request")
            }
            RegisterUserToFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RegisterUserToFormationError::UnknownFormation => {
                write!(f, "Unknown formation")
            }
            RegisterUserToFormationError::UnknownUser => {
                write!(f, "Unknown user")
            }
        }
    }
}

impl ResponseError for RegisterUserToFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            RegisterUserToFormationError::BadRequest => StatusCode::BAD_REQUEST,
            RegisterUserToFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RegisterUserToFormationError::UnknownFormation => StatusCode::NOT_FOUND,
            RegisterUserToFormationError::UnknownUser => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_register_user_to_formation(
    state: web::Data<AppState>,
    _user_id: u64,
    view: RegisterUserView,
    formation_id: u64,
) -> Result<(), RegisterUserToFormationError> {
    let register_view = RegisterUserToFormationQueryView::new(view.user_id(), formation_id);
    let outcome: RegisterUserToFormationRow = state
        .get_smart_db()
        .fetch_one(&register_view)
        .await
        .map_err(|err| match err {
            // The user or the course was deleted by a concurrent transaction.
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                RegisterUserToFormationError::UnknownFormation
            }
            err => log_error(
                "trigger_register_user_to_formation",
                RegisterUserToFormationError::DatabaseError,
            )(err),
        })?;

    if !outcome.user_exists() {
        return Err(RegisterUserToFormationError::UnknownUser);
    }
    if !outcome.course_exists() {
        return Err(RegisterUserToFormationError::UnknownFormation);
    }
    Ok(())
}

#[utoipa::path(
    post,
    params(
        AdminFormationIdParams,
    ),
    path = "/",
    summary = "Enrol an agent in a formation",
    description = "Enrols a user in a formation. It is the only way to enrol someone: an \
                   agent cannot enrol themselves. The formation then shows up in their \
                   `GET /api/v1/formations/`, with the `NotStarted` status.\n\n\
                   The expected id is the one of the account in Core API. The existence checks \
                   and the enrolment run as a single statement.\n\n\
                   Idempotent: enrolling an agent who is already enrolled answers `200` and \
                   keeps their progress. The response has an empty body.\n\n\
                   For the inverse operation, see \
                   `DELETE /api/v1/admin/users/{user_id}/{formation_id}/`.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Agent enrolled in the formation, or already enrolled. Empty body.",
        ),
        (
            status = 400,
            description = "Malformed JSON body, non-integer `formation_id`, or missing `user_id` field.",
            body = String,
            content_type = "text/plain",
            example = json!("Json deserialize error: missing field `user_id`")
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
            status = 403,
            description = "The caller is authenticated but is not an admin (checked by `AdminMiddleware` before the handler runs).",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden: User is not an admin.")
        ),
        (
            status = 404,
            description = "The formation or the user does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown formation")
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
    request_body(
        content = RegisterUserView,
        description = "Core API id of the agent to enrol.",
        example = json!({ "user_id": 42 })
    ),
    security(
        ("jwt" = [])
    )
)]
#[post("/")]
pub async fn register_user_to_formation(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    request_view: web::Json<RegisterUserView>,
    params: web::Path<AdminFormationIdParams>,
) -> Result<impl Responder, RegisterUserToFormationError> {
    let view = match request_view.try_into() {
        Ok(view) => view,
        Err(_) => return Err(RegisterUserToFormationError::BadRequest),
    };
    trigger_register_user_to_formation(state, auth_user.id, view, params.formation_id).await?;
    Ok(HttpResponse::Ok())
}
