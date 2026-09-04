use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::complete_module::view::CompleteModuleQueryView;
use crate::endpoints::v1::formations::formation_id::module_id::ModuleIdParams;

#[derive(Debug, Clone, PartialEq)]
pub enum CompleteModuleError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for CompleteModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompleteModuleError::BadRequest => {
                write!(f, "Bad request")
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
            CompleteModuleError::BadRequest => StatusCode::BAD_REQUEST,
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
    module_id: u64,
) -> Result<(), CompleteModuleError> {
    let view = CompleteModuleQueryView::new(user_id, module_id);
    state.get_smart_db().execute(view).await.map_err(|err| {
        if matches!(err, ApiLibError::Database(DbError::ForeignKeyViolation(_))) {
            // `module_id` doesn't exist.
            CompleteModuleError::BadRequest
        } else {
            CompleteModuleError::DatabaseError
        }
    })
}

#[utoipa::path(
    params(
        ModuleIdParams,
    ),
    patch,
    path = "",
    responses(
        (status = 200, description = "Module completed successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
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
    trigger_complete_module(state, auth_user.id, params.module_id).await?;
    Ok(HttpResponse::Ok())
}
