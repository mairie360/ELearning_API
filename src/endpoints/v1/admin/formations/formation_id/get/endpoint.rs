use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::get_formation_modules::view::{
    AdminFormationModuleRow, AdminModuleContentRow, GetFormationModulesQueryView,
};
use crate::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use crate::endpoints::v1::admin::formations::formation_id::get::view::GetFormationByIdResultView;
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationIdParams;
use crate::endpoints::v1::admin::formations::{AdminFormationModule, AdminModuleContent};
use crate::endpoints::v1::admin::AdminUserDetailsQuery;

fn map_content(row: AdminModuleContentRow) -> AdminModuleContent {
    AdminModuleContent::new(row.id() as u64, row.file_name(), row.file_type())
}

fn map_module(row: AdminFormationModuleRow) -> AdminFormationModule {
    AdminFormationModule::new(
        row.id() as u64,
        row.name(),
        row.description().unwrap_or_default(),
        row.content()
            .map(|content| content.iter().cloned().map(map_content).collect()),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetFormationByIdError {
    BadRequest,
    DatabaseError,
    NotFound,
}

impl std::fmt::Display for GetFormationByIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetFormationByIdError::BadRequest => {
                write!(f, "Bad request.")
            }
            GetFormationByIdError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetFormationByIdError::NotFound => {
                write!(f, "The formation was not found.")
            }
        }
    }
}

impl ResponseError for GetFormationByIdError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetFormationByIdError::BadRequest => StatusCode::BAD_REQUEST,
            GetFormationByIdError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetFormationByIdError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_formation_by_id(
    state: web::Data<AppState>,
    formation_id: u64,
    details: bool,
) -> Result<GetFormationByIdResultView, GetFormationByIdError> {
    let smart_db = state.get_smart_db();

    let exists_view = DoesCourseExistQueryView::new(formation_id);
    let exists: bool = smart_db
        .fetch_scalar(&exists_view)
        .await
        .map_err(|_| GetFormationByIdError::DatabaseError)?;
    if !exists {
        return Err(GetFormationByIdError::NotFound);
    }

    let view = GetFormationModulesQueryView::new(formation_id, details);
    let rows: Vec<AdminFormationModuleRow> = smart_db
        .fetch_all(&view)
        .await
        .map_err(|_| GetFormationByIdError::DatabaseError)?;

    let modules = rows.into_iter().map(map_module).collect();

    Ok(GetFormationByIdResultView { modules })
}

#[utoipa::path(
    get,
    params(
        AdminUserDetailsQuery,
        AdminFormationIdParams,
    ),
    path = "/",
    summary = "Lister les modules d'une formation du catalogue",
    description = "Renvoie les modules d'une formation, sans rapport avec la progression de qui \
                   que ce soit : c'est la structure du cours, pas son suivi. Pour la progression \
                   d'un agent donné, voir `GET /api/v1/admin/users/{user_id}/{formation_id}/`.\n\n\
                   `details=true` fait descendre la réponse jusqu'aux pièces jointes de chaque \
                   module. Sans ce paramètre, `content` est `null` et non un tableau vide.\n\n\
                   Aucun contrôle de rôle n'est appliqué sur le préfixe `/admin` : tout utilisateur \
                   authentifié peut appeler cette route.",
    responses(
        (
            status = 200,
            description = "Modules de la formation.",
            body = GetFormationByIdResultView,
            example = json!({
                "modules": [
                    {
                        "id": 11,
                        "name": "Les principes du RGPD",
                        "description": "Licéité, minimisation, durée de conservation",
                        "content": [
                            { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "pdf" }
                        ]
                    }
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
            status = 404,
            description = "Aucune formation ne porte cet identifiant.",
            body = String,
            content_type = "text/plain",
            example = json!("The formation was not found.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    tag = "Admin - Formations",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_formation_by_id(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    query: web::Query<AdminUserDetailsQuery>,
    params: web::Path<AdminFormationIdParams>,
) -> Result<impl Responder, GetFormationByIdError> {
    let formations =
        trigger_get_formation_by_id(state, params.formation_id, query.details()).await?;
    Ok(HttpResponse::Ok().json(formations))
}
