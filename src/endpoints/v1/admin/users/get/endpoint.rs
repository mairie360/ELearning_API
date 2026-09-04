use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use crate::endpoints::v1::admin::users::get::view::{GetUsersResultView, User};

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
    let view = GetUsersQueryView::new();
    let rows: Vec<UserRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetUsersError::DatabaseError)?;

    let users = rows
        .into_iter()
        .map(|row| User {
            id: row.id() as u64,
            name: row.name().to_string(),
        })
        .collect();

    Ok(GetUsersResultView { users })
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
