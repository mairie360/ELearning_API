use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_formation_modules::view::{
    FormationModuleRow, GetMyFormationModulesQueryView,
};
use crate::endpoints::v1::formations::formation_id::get::view::{GetFormationResponseView, Module};
use crate::endpoints::v1::formations::formation_id::FormationIdParams;

fn map_module(row: FormationModuleRow) -> Module {
    Module {
        id: row.id() as u64,
        name: row.name().to_string(),
        description: row.description().unwrap_or_default().to_string(),
        completed: row.completed(),
    }
}

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
    let view = GetMyFormationModulesQueryView::new(formation_id, user_id);
    let rows: Vec<FormationModuleRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetMeFormationByIdError::DatabaseError)?;

    let modules = rows.into_iter().map(map_module).collect();

    Ok(GetFormationResponseView { modules })
}

#[utoipa::path(
    get,
    params(FormationIdParams),
    path = "",
    summary = "Lister les modules d'une de ses formations",
    description = "Renvoie les modules d'une formation avec, pour chacun, l'indicateur \
                   `completed` propre à l'utilisateur porté par le JWT.\n\n\
                   La progression renvoyée est toujours celle de l'appelant. Pour consulter celle \
                   d'un autre agent, passer par \
                   `GET /api/v1/admin/users/{user_id}/{formation_id}/`.\n\n\
                   Une formation à laquelle l'appelant n'est pas inscrit renvoie une liste de \
                   modules vide, et non une erreur.",
    responses(
        (
            status = 200,
            description = "Modules de la formation et leur état d'achèvement pour l'appelant.",
            body = GetFormationResponseView,
            example = json!({
                "modules": [
                    { "id": 11, "name": "Les principes du RGPD", "description": "Licéité, minimisation, durée de conservation", "completed": true },
                    { "id": 12, "name": "Gérer une demande d'accès", "description": "Procédure et délais", "completed": false }
                ]
            })
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
