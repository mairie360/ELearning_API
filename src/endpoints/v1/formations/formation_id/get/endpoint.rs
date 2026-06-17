use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::formations::formation_id::get::view::GetFormationResponseView;
use crate::endpoints::v1::formations::formation_id::FormationIdParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetMeFormationByIdError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetMeFormationByIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetMeFormationByIdError::BadRequest => {
                write!(f, "Bad request.")
            }
            GetMeFormationByIdError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetMeFormationByIdError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetMeFormationByIdError::BadRequest => StatusCode::BAD_REQUEST,
            GetMeFormationByIdError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_my_formation_by_id(
    state: web::Data<AppState>,
    formation_id: u64,
    user_id: u64,
) -> Result<GetFormationResponseView, GetMeFormationByIdError> {
    //get_cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetMeFormationByIdError::DatabaseError),
    };

    //query

    // update cache

    Ok(GetFormationResponseView { modules: vec![] })
}

#[utoipa::path(
    get,
    params(FormationIdParams),
    path = "",
    responses(
        (status = 200, description = "Formation retrieved successfully", body = GetFormationResponseView),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Formations",
)]
#[get("/")]
pub async fn get_my_formation_by_id(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<FormationIdParams>,
) -> Result<impl Responder, GetMeFormationByIdError> {
    let params = params.into_inner();
    let formation =
        trigger_get_my_formation_by_id(state, params.formation_id, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(formation))
}
