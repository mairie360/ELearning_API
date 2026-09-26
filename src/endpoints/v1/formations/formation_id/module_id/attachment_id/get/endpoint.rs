use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_attachment::view::{AttachmentRow, GetAttachmentQueryView};
use crate::endpoints::v1::formations::formation_id::module_id::access::{
    check_module_access, ModuleAccessError,
};
use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::get::view::GetAttachmentUrlView;
use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::AttachmentIdParams;
use crate::storage::{mime_for, FileStorage};

#[derive(Debug, Clone, PartialEq)]
pub enum GetAttachmentUrlError {
    Forbidden,
    NotFound,
    DatabaseError,
    StorageError,
}

impl From<ModuleAccessError> for GetAttachmentUrlError {
    fn from(err: ModuleAccessError) -> Self {
        match err {
            ModuleAccessError::NotEnrolled => GetAttachmentUrlError::Forbidden,
            ModuleAccessError::ModuleNotFound => GetAttachmentUrlError::NotFound,
            ModuleAccessError::DatabaseError => GetAttachmentUrlError::DatabaseError,
        }
    }
}

impl std::fmt::Display for GetAttachmentUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetAttachmentUrlError::Forbidden => {
                write!(f, "You are not enrolled in this formation.")
            }
            GetAttachmentUrlError::NotFound => write!(f, "The attachment was not found."),
            GetAttachmentUrlError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetAttachmentUrlError::StorageError => {
                write!(f, "An error occurred while accessing the file storage.")
            }
        }
    }
}

impl ResponseError for GetAttachmentUrlError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetAttachmentUrlError::Forbidden => StatusCode::FORBIDDEN,
            GetAttachmentUrlError::NotFound => StatusCode::NOT_FOUND,
            GetAttachmentUrlError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetAttachmentUrlError::StorageError => StatusCode::BAD_GATEWAY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_attachment_url(
    state: web::Data<AppState>,
    storage: &dyn FileStorage,
    user_id: u64,
    formation_id: u64,
    module_id: u64,
    attachment_id: u64,
) -> Result<GetAttachmentUrlView, GetAttachmentUrlError> {
    check_module_access(state.get_smart_db(), user_id, formation_id, module_id).await?;

    let view = GetAttachmentQueryView::new(formation_id, module_id, attachment_id);
    let rows: Vec<AttachmentRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetAttachmentUrlError::DatabaseError)?;

    let attachment = rows
        .into_iter()
        .next()
        .ok_or(GetAttachmentUrlError::NotFound)?;

    let url = storage
        .presigned_view_url(attachment.file_url(), mime_for(attachment.file_type()))
        .await
        .map_err(|_| GetAttachmentUrlError::StorageError)?;

    Ok(GetAttachmentUrlView::new(url))
}

#[utoipa::path(
    get,
    params(
        AttachmentIdParams,
    ),
    path = "",
    summary = "Get the URL of an attachment",
    description = "Returns a short-lived signed URL that displays the attachment directly in the \
                   browser. The file itself never goes through the API.\n\n\
                   The URL is meant for immediate use: request it again on every opening rather \
                   than storing it, since it expires. The client does not need to know that it \
                   points to an object storage.\n\n\
                   Only available to a caller enrolled in the formation (`403` otherwise, admins \
                   included: an admin enrols through \
                   `POST /api/v1/admin/formations/{formation_id}/`).\n\n\
                   Note the `502`, specific to this endpoint: the file storage is unreachable or \
                   refuses to sign the URL although the attachment exists in the database. Not to \
                   be confused with the `404`, which means the attachment does not exist in this \
                   module of this formation.",
    responses(
        (
            status = 200,
            description = "Signed URL of the attachment.",
            body = GetAttachmentUrlView,
            example = json!({
                "url": "https://storage.mairie360.fr/elearning/rgpd-principes.pdf?X-Amz-Expires=900&X-Amz-Signature=..."
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
            description = "The caller is not enrolled in this formation (or the formation does not exist). No URL is signed.",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
        ),
        (
            status = 404,
            description = "The module does not belong to this formation, or no attachment with this id exists in this module.",
            body = String,
            content_type = "text/plain",
            example = json!("The attachment was not found.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
        (
            status = 502,
            description = "The file storage is unreachable or refused to sign the URL. The attachment exists but its URL could not be produced: the call can be retried.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the file storage.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Formations",
)]
#[get("/")]
pub async fn get_attachment_url(
    state: web::Data<AppState>,
    storage: web::Data<dyn FileStorage>,
    auth_user: AuthenticatedUser,
    params: web::Path<AttachmentIdParams>,
) -> Result<impl Responder, GetAttachmentUrlError> {
    let params = params.into_inner();
    let attachment = trigger_get_attachment_url(
        state,
        storage.get_ref(),
        auth_user.id,
        params.formation_id,
        params.module_id,
        params.attachment_id,
    )
    .await?;
    Ok(HttpResponse::Ok().json(attachment))
}
