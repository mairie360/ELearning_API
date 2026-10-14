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
use crate::logging::log_error;

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
    summary = "Lister ses formations avec leurs modules et leurs fichiers",
    description = "Renvoie, en un seul appel, les formations auxquelles l'utilisateur porté par le \
                   JWT est inscrit, avec pour chacune son avancement, ses modules (et si \
                   l'utilisateur les a terminés) et les fichiers de chaque module (nom, type, taille).\n\n\
                   C'est la réunion de `GET /api/v1/formations/`, de `GET /api/v1/formations/{formation_id}/` \
                   et de `GET /api/v1/formations/{formation_id}/{module_id}/` pour toutes les formations de \
                   l'utilisateur, calculée en un nombre fixe de requêtes quel que soit le nombre de formations \
                   ou de modules. Mêmes règles d'accès que ces routes : seules les inscriptions de l'appelant \
                   sont visibles, admins compris.\n\n\
                   Le contenu des fichiers n'est **pas** renvoyé : pour en ouvrir un, demander une URL signée à \
                   `GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/`.",
    responses(
        (
            status = 200,
            description = "Formations de l'utilisateur connecté avec leurs modules et leurs fichiers. Liste vide s'il n'est inscrit à aucune formation.",
            body = GetCatalogResultView,
            example = json!({
                "formations": [
                    {
                        "id": 4,
                        "name": "RGPD pour les agents territoriaux",
                        "description": "Obligations et bonnes pratiques",
                        "status": "InProgress",
                        "modules": [
                            {
                                "id": 11,
                                "name": "Les principes du RGPD",
                                "description": "Licéité, minimisation, durée de conservation",
                                "completed": true,
                                "files": [
                                    { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "Pdf", "file_size_bytes": 482913 }
                                ]
                            },
                            {
                                "id": 12,
                                "name": "Les droits des personnes",
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
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
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
#[get("/catalog/")]
pub async fn get_my_catalog(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetCatalogError> {
    let catalog = trigger_get_catalog(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(catalog))
}
