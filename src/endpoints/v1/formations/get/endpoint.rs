use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use crate::endpoints::v1::formations::get::view::{Formation, GetFormationsResultView, Status};

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
    let view = GetMyFormationsQueryView::new(user_id);
    let rows: Vec<FormationSummaryRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetFormationsError::DatabaseError)?;

    let formations = rows
        .into_iter()
        .map(|row| {
            Formation::new(
                row.id() as u64,
                row.name(),
                row.description().unwrap_or_default(),
                Status::from(row.status().to_string()),
            )
        })
        .collect();

    Ok(GetFormationsResultView::new(formations))
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
