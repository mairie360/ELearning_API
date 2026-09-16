use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_module_attachments::view::{
    GetModuleAttachmentsQueryView, ModuleAttachmentRow,
};
use crate::endpoints::v1::formations::formation_id::module_id::get::view::{
    File, FileType, GetModuleResponseView,
};
use crate::endpoints::v1::formations::formation_id::module_id::ModuleIdParams;

fn map_file(row: ModuleAttachmentRow) -> File {
    // `file_url` (the S3 object key) is deliberately not exposed here: clients
    // fetch a viewable URL for a single file via
    // `GET /v1/formations/{formation_id}/{module_id}/{attachment_id}`.
    File {
        id: row.id() as u64,
        file_name: row.file_name().to_string(),
        file_type: FileType::from(row.file_type().to_string()),
        file_size_bytes: row.file_size_bytes(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetModuleError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetModuleError::BadRequest => {
                write!(f, "Bad request.")
            }
            GetModuleError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetModuleError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetModuleError::BadRequest => StatusCode::BAD_REQUEST,
            GetModuleError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_module(
    state: web::Data<AppState>,
    formation_id: u64,
    module_id: u64,
    _user_id: u64,
) -> Result<GetModuleResponseView, GetModuleError> {
    let view = GetModuleAttachmentsQueryView::new(formation_id, module_id);
    let rows: Vec<ModuleAttachmentRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetModuleError::DatabaseError)?;

    let files = rows.into_iter().map(map_file).collect();

    Ok(GetModuleResponseView { files })
}

#[utoipa::path(
    get,
    params(
        ModuleIdParams,
    ),
    path = "",
    summary = "Lister les pièces jointes d'un module",
    description = "Renvoie les fichiers pédagogiques d'un module : nom, type et taille.\n\n\
                   Le contenu des fichiers n'est **pas** renvoyé ici : pour l'ouvrir, demander une \
                   URL signée à \
                   `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.\n\n\
                   `file_size_bytes` peut être `null` pour un fichier dont la taille n'a pas été \
                   enregistrée. Un `file_type` valant `Error` signale un type stocké en base que \
                   l'API ne sait pas interpréter.",
    responses(
        (
            status = 200,
            description = "Pièces jointes du module.",
            body = GetModuleResponseView,
            example = json!({
                "files": [
                    { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "Pdf", "file_size_bytes": 482913 },
                    { "id": 32, "file_name": "rgpd-introduction.mp4", "file_type": "Video", "file_size_bytes": null }
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
pub async fn get_module(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ModuleIdParams>,
) -> Result<impl Responder, GetModuleError> {
    let params = params.into_inner();
    let formation =
        trigger_get_module(state, params.formation_id, params.module_id, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(formation))
}
