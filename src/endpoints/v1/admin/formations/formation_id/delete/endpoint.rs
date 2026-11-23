use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::delete_course::view::DeleteCourseQueryView;
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationWriteParams;
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteFormationError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for DeleteFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteFormationError::NotFound => write!(f, "The formation was not found."),
            DeleteFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for DeleteFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteFormationError::NotFound => StatusCode::NOT_FOUND,
            DeleteFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_formation(
    state: web::Data<AppState>,
    formation_id: u64,
) -> Result<(), DeleteFormationError> {
    let deleted: bool = state
        .get_smart_db()
        .fetch_scalar(&DeleteCourseQueryView::new(formation_id))
        .await
        .map_err(log_error(
            "trigger_delete_formation",
            DeleteFormationError::DatabaseError,
        ))?;
    if !deleted {
        return Err(DeleteFormationError::NotFound);
    }
    Ok(())
}

#[utoipa::path(
    delete,
    params(AdminFormationWriteParams),
    path = "/",
    summary = "Delete a formation",
    description = "Deletes a formation for good, with its modules, contents, enrolments, progress, \
                   ratings and reminders. The stored files are not removed from the storage. The \
                   response has an empty body.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 204,
            description = "Formation deleted. Empty body.",
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
#[delete("/")]
pub async fn delete_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminFormationWriteParams>,
) -> Result<impl Responder, DeleteFormationError> {
    trigger_delete_formation(state, params.formation_id()).await?;
    Ok(HttpResponse::NoContent())
}
