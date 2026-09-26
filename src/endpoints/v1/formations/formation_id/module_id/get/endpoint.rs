use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_module_attachments::view::{
    GetModuleAttachmentsQueryView, ModuleAttachmentRow,
};
use crate::endpoints::v1::formations::formation_id::module_id::access::{
    check_module_access, ModuleAccessError,
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
    Forbidden,
    NotFound,
    DatabaseError,
}

impl From<ModuleAccessError> for GetModuleError {
    fn from(err: ModuleAccessError) -> Self {
        match err {
            ModuleAccessError::NotEnrolled => GetModuleError::Forbidden,
            ModuleAccessError::ModuleNotFound => GetModuleError::NotFound,
            ModuleAccessError::DatabaseError => GetModuleError::DatabaseError,
        }
    }
}

impl std::fmt::Display for GetModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetModuleError::BadRequest => {
                write!(f, "Bad request.")
            }
            GetModuleError::Forbidden => {
                write!(f, "You are not enrolled in this formation.")
            }
            GetModuleError::NotFound => {
                write!(f, "The module was not found in this formation.")
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
            GetModuleError::Forbidden => StatusCode::FORBIDDEN,
            GetModuleError::NotFound => StatusCode::NOT_FOUND,
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
    user_id: u64,
) -> Result<GetModuleResponseView, GetModuleError> {
    check_module_access(state.get_smart_db(), user_id, formation_id, module_id).await?;

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
    summary = "List the attachments of a module",
    description = "Returns the learning files of a module: name, type and size.\n\n\
                   The file contents are **not** returned here: to open one, request a signed \
                   URL from `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.\n\n\
                   Only available to a caller enrolled in the formation (`403` otherwise, admins \
                   included: an admin enrols through \
                   `POST /api/v1/admin/formations/{formation_id}/`). The module must belong to \
                   the formation of the path (`404` otherwise).\n\n\
                   `file_size_bytes` can be `null` for a file whose size was not recorded. A \
                   `file_type` of `Error` flags a type stored in the database that the API cannot \
                   interpret. A module without files returns an empty `files` list.",
    responses(
        (
            status = 200,
            description = "Attachments of the module.",
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
            description = "A path segment is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
        ),
        (
            status = 401,
            description = "`Authorization` header missing, or JWT invalid or expired.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller is not enrolled in this formation (or the formation does not exist).",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
        ),
        (
            status = 404,
            description = "The module does not exist or belongs to another formation.",
            body = String,
            content_type = "text/plain",
            example = json!("The module was not found in this formation.")
        ),
        (
            status = 500,
            description = "Database error.",
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
