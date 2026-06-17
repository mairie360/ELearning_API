use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::formations::get::view::GetFormationsResultView;

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

async fn trigger_get_my_formations(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<GetFormationsResultView, GetFormationsError> {
    //get_cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetFormationsError::DatabaseError),
    };

    //query

    // update cache

    Ok(GetFormationsResultView::new(vec![]))
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Formations retrieved successfully", body = GetFormationsResultView),
        (status = 500, description = "Internal server error")
    ),
    tag = "Formations",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_my_formations(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetFormationsError> {
    let formations = trigger_get_my_formations(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(formations))
}
