use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::complete_module::view::CompleteModuleQueryView;
use crate::endpoints::v1::formations::formation_id::module_id::ModuleIdParams;

#[derive(Debug, Clone, PartialEq)]
pub enum CompleteModuleError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for CompleteModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompleteModuleError::BadRequest => {
                write!(f, "Bad request")
            }
            CompleteModuleError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for CompleteModuleError {
    fn status_code(&self) -> StatusCode {
        match self {
            CompleteModuleError::BadRequest => StatusCode::BAD_REQUEST,
            CompleteModuleError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_complete_module(
    state: web::Data<AppState>,
    user_id: u64,
    module_id: u64,
) -> Result<(), CompleteModuleError> {
    let smart_db = state.get_smart_db();

    let view = CompleteModuleQueryView::new(user_id, module_id);
    smart_db.execute(view).await.map_err(|err| match err {
        ApiLibError::Database(DbError::ForeignKeyViolation(_)) => CompleteModuleError::BadRequest,
        _ => CompleteModuleError::DatabaseError,
    })?;

    Ok(())
}

#[utoipa::path(
    params(
        ModuleIdParams,
    ),
    patch,
    path = "",
    summary = "Marquer un module comme terminé",
    description = "Enregistre l'achèvement d'un module pour l'utilisateur porté par le JWT. C'est \
                   la seule opération d'écriture qu'un agent peut faire sur sa propre \
                   progression.\n\n\
                   Le statut de la formation est recalculé automatiquement en base : elle passe à \
                   `InProgress` au premier module terminé, puis à `Completed` une fois tous ses \
                   modules achevés. Il n'y a pas d'opération inverse pour « dé-terminer » un \
                   module.\n\n\
                   Opération idempotente : un module déjà terminé répond également `200`. La \
                   réponse a un corps vide.",
    responses(
        (
            status = 200,
            description = "Module marqué comme terminé, ou déjà terminé. Corps vide.",
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
#[patch("/")]
pub async fn complete_module(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ModuleIdParams>,
) -> Result<impl Responder, CompleteModuleError> {
    trigger_complete_module(state, auth_user.id, params.module_id).await?;
    Ok(HttpResponse::Ok())
}
