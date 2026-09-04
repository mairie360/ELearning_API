use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::admin::users::user_id::formation_id::get::view::GetUserFormation;
use crate::endpoints::v1::admin::users::user_id::formation_id::AdminUserFormationIdParams;
use crate::endpoints::v1::admin::AdminUserDetailsQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetUserFormationError {
    DatabaseError,
    UnknowModule,
}

impl std::fmt::Display for GetUserFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUserFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetUserFormationError::UnknowModule => {
                write!(f, "Module not found.")
            }
        }
    }
}

impl ResponseError for GetUserFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUserFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetUserFormationError::UnknowModule => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_user_formation(
    state: web::Data<AppState>,
    module_id: u64,
    user_id: u64,
) -> Result<GetUserFormation, GetUserFormationError> {
    //get_cache

    let _smart_db = state.get_smart_db();

    //query

    // update cache

    Ok(GetUserFormation { modules: vec![] })
}

#[utoipa::path(
    get,
    params(AdminUserDetailsQuery, AdminUserFormationIdParams),
    path = "",
    responses(
        (status = 200, description = "Users with formations retrieved successfully", body = GetUserFormation),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Module not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Users",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_user_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminUserFormationIdParams>,
    _: web::Query<AdminUserDetailsQuery>,
) -> Result<impl Responder, GetUserFormationError> {
    let module = trigger_get_user_formation(state, params.formation_id, params.user_id).await?;
    Ok(HttpResponse::Ok().json(module))
}
