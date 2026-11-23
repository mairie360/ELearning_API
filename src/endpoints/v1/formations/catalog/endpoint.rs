use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_catalog::view::{
    CatalogFileRow, CatalogFormationRow, CatalogModuleRow, GetMyCatalogQueryView,
};
use crate::endpoints::v1::formations::catalog::view::{
    CatalogFormation, CatalogModule, GetCatalogResultView,
};
use crate::endpoints::v1::formations::formation_id::module_id::get::view::{File, FileType};
use crate::endpoints::v1::formations::get::view::Status;
use crate::error::log_error;

fn map_file(row: &CatalogFileRow) -> File {
    File {
        id: id_from_sql(row.id()),
        file_name: row.file_name().to_string(),
        file_type: FileType::from(row.file_type().to_string()),
        file_size_bytes: row.file_size_bytes(),
    }
}

fn map_module(row: &CatalogModuleRow) -> CatalogModule {
    CatalogModule {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
        description: row.description().unwrap_or_default().to_string(),
        completed: row.completed(),
        files: row.files().iter().map(map_file).collect(),
    }
}

fn map_formation(row: CatalogFormationRow) -> CatalogFormation {
    CatalogFormation {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
        description: row.description().unwrap_or_default().to_string(),
        deadline: row.deadline(),
        status: Status::from(row.status().to_string()),
        modules: row.modules().iter().map(map_module).collect(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetCatalogError {
    DatabaseError,
}

impl std::fmt::Display for GetCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCatalogError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetCatalogError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetCatalogError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_catalog(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<GetCatalogResultView, GetCatalogError> {
    let view = GetMyCatalogQueryView::new(user_id);
    let rows: Vec<CatalogFormationRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_catalog",
                GetCatalogError::DatabaseError,
            ))?;

    Ok(GetCatalogResultView {
        formations: rows.into_iter().map(map_formation).collect(),
    })
}

#[utoipa::path(
    get,
    path = "",
    summary = "List one's formations with their modules and files",
    description = "Returns, in one call, the formations the user of the JWT is enrolled in, \
                   with for each its progress, its modules (and whether the user completed \
                   them) and the files of each module (name, type, size).\n\n\
                   It merges `GET /api/v1/formations/`, `GET /api/v1/formations/{formation_id}/` \
                   and `GET /api/v1/formations/{formation_id}/{module_id}/` for every formation of \
                   the user, computed in a fixed number of queries whatever the number of formations \
                   or modules. Same access rules as these routes: only the caller's enrolments \
                   are visible, admins included.\n\n\
                   The content of the files is **not** returned: to open one, ask for a signed URL from \
                   `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.",
    responses(
        (
            status = 200,
            description = "Formations of the signed-in user with their modules and files. Empty list if they are enrolled in no formation.",
            body = GetCatalogResultView,
            example = json!({
                "formations": [
                    {
                        "id": 4,
                        "name": "GDPR for local government staff",
                        "description": "Obligations and good practices",
                        "deadline": "2026-12-15",
                        "status": "InProgress",
                        "modules": [
                            {
                                "id": 11,
                                "name": "The principles of the GDPR",
                                "description": "Lawfulness, minimisation, retention period",
                                "completed": true,
                                "files": [
                                    { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "Pdf", "file_size_bytes": 482913 }
                                ]
                            },
                            {
                                "id": 12,
                                "name": "The rights of individuals",
                                "description": "",
                                "completed": false,
                                "files": []
                            }
                        ]
                    }
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
#[get("/catalog/")]
pub async fn get_my_catalog(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetCatalogError> {
    let catalog = trigger_get_catalog(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(catalog))
}
