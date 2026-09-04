use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::admin::users::get::view::GetUsersResultView;

#[derive(Debug, Clone, PartialEq)]
pub enum GetUsersError {
    DatabaseError,
}

impl std::fmt::Display for GetUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_users(
    state: web::Data<AppState>,
) -> Result<GetUsersResultView, GetUsersError> {
    //get_cache

    let _smart_db = state.get_smart_db();

    //query

    // update cache

    Ok(GetUsersResultView { users: vec![] })
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Users with formations retrieved successfully", body = GetUsersResultView),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Users",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_users(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
) -> Result<impl Responder, GetUsersError> {
    let formations = trigger_get_users(state).await?;
    Ok(HttpResponse::Ok().json(formations))
}
