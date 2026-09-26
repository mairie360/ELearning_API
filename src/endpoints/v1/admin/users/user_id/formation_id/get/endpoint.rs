use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_user_formation::view::{
    GetUserFormationQueryView, UserFormationModuleRow, UserModuleContentRow,
};
use crate::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use crate::endpoints::v1::admin::users::user_id::formation_id::get::view::GetUserFormation;
use crate::endpoints::v1::admin::users::user_id::formation_id::AdminUserFormationIdParams;
use crate::endpoints::v1::admin::users::{UsersFormationModule, UsersModuleContent};
use crate::endpoints::v1::admin::AdminUserDetailsQuery;

fn map_content(row: UserModuleContentRow) -> UsersModuleContent {
    UsersModuleContent::new(row.id() as u64, row.file_name(), row.file_type(), None)
}

fn map_module(row: UserFormationModuleRow) -> UsersFormationModule {
    UsersFormationModule::new(
        row.id() as u64,
        row.name(),
        row.description().unwrap_or_default(),
        row.content().iter().cloned().map(map_content).collect(),
        row.is_completed(),
        row.completed_at().map(|dt| dt.and_utc()),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetUserFormationError {
    DatabaseError,
    UnknowModule,
}

impl std::fmt::Display for GetUserFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUserFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetUserFormationError::UnknowModule => {
                write!(f, "Module not found.")
            }
        }
    }
}

impl ResponseError for GetUserFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUserFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetUserFormationError::UnknowModule => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_user_formation(
    state: web::Data<AppState>,
    formation_id: u64,
    user_id: u64,
    details: bool,
) -> Result<GetUserFormation, GetUserFormationError> {
    let smart_db = state.get_smart_db();

    let exists_view = DoesCourseExistQueryView::new(formation_id);
    let exists: bool = smart_db
        .fetch_scalar(&exists_view)
        .await
        .map_err(|_| GetUserFormationError::DatabaseError)?;
    if !exists {
        return Err(GetUserFormationError::UnknowModule);
    }

    let view = GetUserFormationQueryView::new(formation_id, user_id, details);
    let rows: Vec<UserFormationModuleRow> = smart_db
        .fetch_all(&view)
        .await
        .map_err(|_| GetUserFormationError::DatabaseError)?;

    let modules = rows.into_iter().map(map_module).collect();

    Ok(GetUserFormation { modules })
}

#[utoipa::path(
    get,
    params(AdminUserDetailsQuery, AdminUserFormationIdParams),
    path = "",
    summary = "Consulter la progression d'un agent sur une formation",
    description = "Renvoie les modules d'une formation avec, pour chacun, l'état d'achèvement de \
                   l'agent visé et la date à laquelle il l'a terminé. Vue la plus fine du suivi : \
                   elle descend jusqu'à la date de consultation de chaque pièce jointe.\n\n\
                   `completed_at` et `finished_at` sont `null` tant que l'élément n'a pas été \
                   terminé.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Modules de la formation et progression de l'agent sur chacun.",
            body = GetUserFormation,
            example = json!({
                "modules": [
                    {
                        "id": 11,
                        "name": "Les principes du RGPD",
                        "description": "Licéité, minimisation, durée de conservation",
                        "content": [
                            { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "pdf", "finished_at": "2026-09-03T14:22:00Z" }
                        ],
                        "is_completed": true,
                        "completed_at": "2026-09-03T14:25:00Z"
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
            description = "`Authorization` header missing, or JWT invalid or expired.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller is authenticated but is not an admin (checked by `AdminMiddleware` before the handler runs).",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden: User is not an admin.")
        ),
        (
            status = 404,
            description = "Aucun module trouvé pour ce couple agent / formation.",
            body = String,
            content_type = "text/plain",
            example = json!("Module not found.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    tag = "Admin - Users",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_user_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminUserFormationIdParams>,
    query: web::Query<AdminUserDetailsQuery>,
) -> Result<impl Responder, GetUserFormationError> {
    let module =
        trigger_get_user_formation(state, params.formation_id, params.user_id, query.details())
            .await?;
    Ok(HttpResponse::Ok().json(module))
}
