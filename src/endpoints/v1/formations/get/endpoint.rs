use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use crate::endpoints::v1::formations::get::view::{Formation, GetFormationsResultView, Status};

fn map_formation(row: FormationSummaryRow) -> Formation {
    Formation::new(
        row.id() as u64,
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
    let rows: Vec<FormationSummaryRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetFormationsError::DatabaseError)?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetFormationsResultView::new(formations))
}

#[utoipa::path(
    get,
    path = "",
    summary = "Lister ses formations",
    description = "Renvoie les formations auxquelles l'utilisateur porté par le JWT est inscrit, \
                   avec l'avancement de chacune. Seul un administrateur peut l'y inscrire, via \
                   `POST /api/v1/admin/formations/{formation_id}/` : un agent ne peut pas s'inscrire \
                   lui-même, et cette liste est donc vide tant qu'on ne l'a pas inscrit.\n\n\
                   Vue de liste : les modules ne sont pas inclus, il faut passer par \
                   `GET /api/v1/formations/{formation_id}/`.",
    responses(
        (
            status = 200,
            description = "Formations de l'utilisateur connecté et leur avancement.",
            body = GetFormationsResultView,
            example = json!({
                "formations": [
                    { "id": 4, "name": "RGPD pour les agents territoriaux", "description": "Obligations et bonnes pratiques", "status": "InProgress" },
                    { "id": 9, "name": "Accueil du public en situation de handicap", "description": "", "status": "NotStarted" }
                ]
            })
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
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
