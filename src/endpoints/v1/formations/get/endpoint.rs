use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use crate::endpoints::v1::formations::get::view::{Formation, GetFormationsResultView, Status};
use crate::error::log_error;

fn map_formation(row: FormationSummaryRow) -> Formation {
    Formation::new(
        id_from_sql(row.id()),
        row.name(),
        row.description().unwrap_or_default(),
        Status::from(row.status().to_string()),
    )
}

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
    let rows: Vec<FormationSummaryRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_my_formations",
                GetFormationsError::DatabaseError,
            ))?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetFormationsResultView::new(formations))
}

#[utoipa::path(
    get,
    path = "",
    summary = "List one's formations",
    description = "Returns the formations the user of the JWT is enrolled in, \
                   with the progress of each. Only an administrator can enrol them, through \
                   `POST /api/v1/admin/formations/{formation_id}/`: an agent cannot enrol \
                   themselves, so this list stays empty until someone enrols them.\n\n\
                   List view: the modules are not included, use \
                   `GET /api/v1/formations/{formation_id}/`.",
    responses(
        (
            status = 200,
            description = "Formations of the signed-in user and their progress.",
            body = GetFormationsResultView,
            example = json!({
                "formations": [
                    { "id": 4, "name": "GDPR for local government staff", "description": "Obligations and good practices", "status": "InProgress" },
                    { "id": 9, "name": "Accueil du public en situation de handicap", "description": "", "status": "NotStarted" }
                ]
            })
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 429,
            description = "The caller exceeded their request quota (per user, `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST`). `Retry-After` gives the seconds to wait.",
            body = String,
            content_type = "text/plain",
            example = json!("Too many requests, retry in 1 s.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
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
