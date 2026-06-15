use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::admin::users::user_id::get::view::GetUserByIdResultView;
use crate::endpoints::v1::admin::users::user_id::AdminUserIdParams;
use crate::endpoints::v1::admin::AdminUserDetailsQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetUserFormationsError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetUserFormationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUserFormationsError::BadRequest => {
                write!(f, "Bad request")
            }
            GetUserFormationsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUserFormationsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUserFormationsError::BadRequest => StatusCode::BAD_REQUEST,
            GetUserFormationsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_user(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<GetUserByIdResultView, GetUserFormationsError> {
    //get_cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetUserFormationsError::DatabaseError),
    };

    //query

    // update cache

    Ok(GetUserByIdResultView { formations: vec![] })
}

#[utoipa::path(
    get,
    params(AdminUserDetailsQuery, AdminUserIdParams),
    path = "",
    responses(
        (status = 200, description = "User formations retrieved successfully", body = GetUserByIdResultView),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Users",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_user_formations(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    _: web::Query<AdminUserDetailsQuery>,
    path_params: web::Path<AdminUserIdParams>,
) -> Result<impl Responder, GetUserFormationsError> {
    let formations = trigger_get_user(state, path_params.user_id).await?;
    Ok(HttpResponse::Ok().json(formations))
}
