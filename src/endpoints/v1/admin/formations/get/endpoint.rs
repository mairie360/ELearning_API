use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::admin::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::admin::AdminUserDetailsQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetFormationsError {
    DatabaseError,
}

impl std::fmt::Display for GetFormationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetFormationsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetFormationsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetFormationsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_formations(
    state: web::Data<AppState>,
) -> Result<GetFormationsResultView, GetFormationsError> {
    //get_cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetFormationsError::DatabaseError),
    };

    //query

    // update cache

    Ok(GetFormationsResultView { formations: vec![] })
}

#[utoipa::path(
    get,
    params(
        AdminUserDetailsQuery,
    ),
    path = "",
    responses(
        (status = 200, description = "Formations retrieved successfully", body = GetFormationsResultView),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Formations",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_formations(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    _: web::Query<AdminUserDetailsQuery>,
) -> Result<impl Responder, GetFormationsError> {
    let formations = trigger_get_formations(state).await?;
    Ok(HttpResponse::Ok().json(formations))
}
