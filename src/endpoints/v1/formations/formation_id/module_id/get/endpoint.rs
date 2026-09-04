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

    let files = rows
        .into_iter()
        .map(|row| File {
            id: row.id() as u64,
            file_name: row.file_name().to_string(),
            file_type: FileType::from(row.file_type().to_string()),
            file_url: row.file_url().to_string(),
            file_size_bytes: row.file_size_bytes(),
        })
        .collect();

    Ok(GetModuleResponseView { files })
}

#[utoipa::path(
    get,
    params(
        ModuleIdParams,
    ),
    path = "",
    responses(
        (status = 200, description = "Formation retrieved successfully", body = GetModuleResponseView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
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
