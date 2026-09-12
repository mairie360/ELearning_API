use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_attachment::view::{AttachmentRow, GetAttachmentQueryView};
use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::get::view::GetAttachmentUrlView;
use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::AttachmentIdParams;
use crate::storage::{mime_for, FileStorage};

#[derive(Debug, Clone, PartialEq)]
pub enum GetAttachmentUrlError {
    NotFound,
    DatabaseError,
    StorageError,
}

impl std::fmt::Display for GetAttachmentUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    formation_id: u64,
    module_id: u64,
    attachment_id: u64,
) -> Result<GetAttachmentUrlView, GetAttachmentUrlError> {
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
    responses(
        (status = 200, description = "Attachment URL generated successfully", body = GetAttachmentUrlView),
        (status = 404, description = "Attachment not found"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "File storage error")
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
    _: AuthenticatedUser,
    params: web::Path<AttachmentIdParams>,
) -> Result<impl Responder, GetAttachmentUrlError> {
    let params = params.into_inner();
    let attachment = trigger_get_attachment_url(
        state,
        storage.get_ref(),
        params.formation_id,
        params.module_id,
        params.attachment_id,
    )
    .await?;
    Ok(HttpResponse::Ok().json(attachment))
}
