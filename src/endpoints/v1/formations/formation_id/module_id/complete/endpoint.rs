use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::complete_module::view::CompleteModuleQueryView;
use crate::endpoints::v1::formations::formation_id::module_id::access::{
    check_module_access, ModuleAccessError,
};
use crate::endpoints::v1::formations::formation_id::module_id::ModuleIdParams;

#[derive(Debug, Clone, PartialEq)]
pub enum CompleteModuleError {
    Forbidden,
    NotFound,
    DatabaseError,
}

impl From<ModuleAccessError> for CompleteModuleError {
    fn from(err: ModuleAccessError) -> Self {
        match err {
            ModuleAccessError::NotEnrolled => CompleteModuleError::Forbidden,
            ModuleAccessError::ModuleNotFound => CompleteModuleError::NotFound,
            ModuleAccessError::DatabaseError => CompleteModuleError::DatabaseError,
        }
    }
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
    let smart_db = state.get_smart_db();

    check_module_access(smart_db, user_id, formation_id, module_id).await?;

    let view = CompleteModuleQueryView::new(user_id, module_id);
    smart_db.execute(view).await.map_err(|err| match err {
        // The module was deleted between the access check and the insert.
        ApiLibError::Database(DbError::ForeignKeyViolation(_)) => CompleteModuleError::NotFound,
        _ => CompleteModuleError::DatabaseError,
    })?;

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
                   The status of the formation is recomputed automatically in the database: it \
                   switches to `InProgress` on the first completed module, then to `Completed` \
                   once all its modules are done. There is no inverse operation to \
                   \"un-complete\" a module.\n\n\
                   Idempotent: an already completed module also answers `200`. The response has \
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
