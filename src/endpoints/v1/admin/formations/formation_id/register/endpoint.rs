use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::admin::formations::formation_id::register::view::RegisterUserView;
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationIdParams;

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
    user_id: u64,
    view: RegisterUserView,
    formation_id: u64,
) -> Result<(), RegisterUserToFormationError> {
    let _smart_db = state.get_smart_db();

    //query

    // cache

    Ok(())
}

#[utoipa::path(
    post,
    params(
        AdminFormationIdParams,
    ),
    path = "",
    responses(
        (status = 200, description = "User registered to formation successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Formations",
    request_body = RegisterUserView,
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
