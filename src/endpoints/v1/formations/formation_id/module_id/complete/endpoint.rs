use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::complete_module::view::{
    CompleteModuleQueryView, CompleteModuleRow,
};
use crate::endpoints::v1::formations::formation_id::module_id::ModuleIdParams;
use crate::logging::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum CompleteModuleError {
    Forbidden,
    NotFound,
    PreviousModulesPending,
    DatabaseError,
}

impl std::fmt::Display for CompleteModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompleteModuleError::Forbidden => {
                write!(f, "You are not enrolled in this formation.")
            }
            CompleteModuleError::NotFound => {
                write!(f, "The module was not found in this formation.")
            }
            CompleteModuleError::PreviousModulesPending => {
                write!(f, "Complete the previous modules of this formation first.")
            }
            CompleteModuleError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for CompleteModuleError {
    fn status_code(&self) -> StatusCode {
        match self {
            CompleteModuleError::Forbidden => StatusCode::FORBIDDEN,
            CompleteModuleError::NotFound => StatusCode::NOT_FOUND,
            CompleteModuleError::PreviousModulesPending => StatusCode::CONFLICT,
            CompleteModuleError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_complete_module(
    state: web::Data<AppState>,
    user_id: u64,
    formation_id: u64,
    module_id: u64,
) -> Result<(), CompleteModuleError> {
    // Access check and write in one statement (MAIR-420): the enrolment
    // cannot be revoked between the check and the upsert.
    let view = CompleteModuleQueryView::new(user_id, formation_id, module_id);
    let outcome: CompleteModuleRow =
        state
            .get_smart_db()
            .fetch_one(&view)
            .await
            .map_err(|err| match err {
                // The module was deleted between the check and the insert.
                ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                    CompleteModuleError::NotFound
                }
                err => log_error(
                    "trigger_complete_module",
                    CompleteModuleError::DatabaseError,
                )(err),
            })?;

    // Same precedence as `check_module_access`: a caller who is not enrolled
    // cannot probe which module ids exist in the formation.
    if !outcome.enrolled() {
        return Err(CompleteModuleError::Forbidden);
    }
    if !outcome.module_in_formation() {
        return Err(CompleteModuleError::NotFound);
    }
    if !outcome.completed() {
        return Err(CompleteModuleError::PreviousModulesPending);
    }
    Ok(())
}

#[utoipa::path(
    params(
        ModuleIdParams,
    ),
    patch,
    path = "",
    summary = "Mark a module as completed",
    description = "Records the completion of a module for the user carried by the JWT. It is \
                   the only write operation an agent can perform on their own progress.\n\n\
                   Only available to a caller enrolled in the formation (`403` otherwise, admins \
                   included). The module must belong to the formation of the path (`404` \
                   otherwise): no progress is recorded in either case.\n\n\
                   Modules are completed in order: every module that comes before this one in \
                   the formation (the order of `GET /api/v1/formations/{formation_id}/`) must \
                   already be completed, otherwise the call answers `409` and records \
                   nothing.\n\n\
                   The status of the formation is recomputed automatically in the database: it \
                   switches to `InProgress` on the first completed module, then to `Completed` \
                   once all its modules are done. There is no inverse operation to \
                   \"un-complete\" a module.\n\n\
                   Idempotent: an already completed module also answers `200`, even if a \
                   previous module is not completed. The response has \
                   an empty body.",
    responses(
        (
            status = 200,
            description = "Module marked as completed, or already completed. Empty body.",
        ),
        (
            status = 400,
            description = "A path segment is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
            description = "The caller is not enrolled in this formation (or the formation does not exist).",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
        ),
        (
            status = 404,
            description = "The module does not exist or belongs to another formation.",
            body = String,
            content_type = "text/plain",
            example = json!("The module was not found in this formation.")
        ),
        (
            status = 409,
            description = "A module that comes before this one in the formation is not completed yet. Nothing is recorded.",
            body = String,
            content_type = "text/plain",
            example = json!("Complete the previous modules of this formation first.")
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
#[patch("/")]
pub async fn complete_module(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ModuleIdParams>,
) -> Result<impl Responder, CompleteModuleError> {
    trigger_complete_module(state, auth_user.id, params.formation_id, params.module_id).await?;
    Ok(HttpResponse::Ok())
}
