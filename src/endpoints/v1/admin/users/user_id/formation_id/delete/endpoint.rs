use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::purge_user_formation_progress::view::PurgeUserFormationProgressQueryView;
use crate::database::admin::users::unsub_user_formation::view::{
    UnsubUserFormationQueryView, UnsubUserFormationRow,
};
use crate::endpoints::v1::admin::users::user_id::formation_id::AdminUserFormationIdParams;
use crate::logging::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum UnsubFormationError {
    DatabaseError,
    NotEnrolled,
}

impl std::fmt::Display for UnsubFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnsubFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            UnsubFormationError::NotEnrolled => {
                write!(f, "The user is not enrolled in this formation.")
            }
        }
    }
}

impl ResponseError for UnsubFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            UnsubFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            UnsubFormationError::NotEnrolled => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_unsub_formation(
    state: web::Data<AppState>,
    formation_id: u64,
    user_id: u64,
) -> Result<(), UnsubFormationError> {
    let db_error = || {
        log_error(
            "trigger_unsub_formation",
            UnsubFormationError::DatabaseError,
        )
    };

    // Two statements in one transaction (MAIR-420), see
    // `UnsubUserFormationQueryView`. Returning early drops the transaction,
    // which rolls it back.
    let mut tx = state.get_smart_db().begin().await.map_err(db_error())?;

    let view = UnsubUserFormationQueryView::new(user_id, formation_id);
    let outcome: UnsubUserFormationRow = tx.fetch_one(&view).await.map_err(db_error())?;
    if !outcome.unregistered() {
        return Err(UnsubFormationError::NotEnrolled);
    }

    let purge = PurgeUserFormationProgressQueryView::new(user_id, formation_id);
    tx.execute(&purge).await.map_err(db_error())?;
    tx.commit().await.map_err(db_error())?;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "/",
    summary = "Unenrol an agent from a formation",
    description = "Removes the enrolment of an agent in a formation. Inverse operation of \
                   `POST /api/v1/admin/formations/{formation_id}/`.\n\n\
                   The progress of the agent on this formation (completed modules) is deleted \
                   with the enrolment, in the same transaction; their progress on other \
                   formations is untouched. Enrolling the agent again starts from scratch, with \
                   the `NotStarted` status.\n\n\
                   Not idempotent: unenrolling an agent who is not enrolled (or from an unknown \
                   formation) answers `404`.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 204,
            description = "Agent unenrolled and their progress on the formation deleted. Empty body.",
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
            description = "The agent is not enrolled in this formation, or the formation does not exist. Nothing was deleted.",
            body = String,
            content_type = "text/plain",
            example = json!("The user is not enrolled in this formation.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    tag = "Admin - Users",
    params(
        AdminUserFormationIdParams
    ),
    security(
        ("jwt" = [])
    )
)]
#[delete("/")]
pub async fn unsub_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminUserFormationIdParams>,
) -> Result<impl Responder, UnsubFormationError> {
    trigger_unsub_formation(state, params.formation_id, params.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
