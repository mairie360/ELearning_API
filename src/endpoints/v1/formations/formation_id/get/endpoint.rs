use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::formations::get_my_formation_modules::view::{
    FormationModuleRow, GetMyFormationModulesQueryView,
};
use crate::database::formations::is_enrolled::view::IsEnrolledQueryView;
use crate::endpoints::v1::formations::formation_id::get::view::{GetFormationResponseView, Module};
use crate::endpoints::v1::formations::formation_id::FormationIdParams;

fn map_module(row: FormationModuleRow) -> Module {
    Module {
        id: row.id() as u64,
        name: row.name().to_string(),
        description: row.description().unwrap_or_default().to_string(),
        completed: row.completed(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetMeFormationByIdError {
    BadRequest,
    Forbidden,
    DatabaseError,
}

impl std::fmt::Display for GetMeFormationByIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetMeFormationByIdError::BadRequest => {
                write!(f, "Bad request.")
            }
            GetMeFormationByIdError::Forbidden => {
                write!(f, "You are not enrolled in this formation.")
            }
            GetMeFormationByIdError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetMeFormationByIdError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetMeFormationByIdError::BadRequest => StatusCode::BAD_REQUEST,
            GetMeFormationByIdError::Forbidden => StatusCode::FORBIDDEN,
            GetMeFormationByIdError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_my_formation_by_id(
    state: web::Data<AppState>,
    formation_id: u64,
    user_id: u64,
) -> Result<GetFormationResponseView, GetMeFormationByIdError> {
    let smart_db = state.get_smart_db();

    // Same rule as the module guard (`module_id::access`): an unknown formation
    // is indistinguishable from one the caller is not enrolled in (`403` for
    // both), so the route never reveals which formation ids exist.
    let enrolled: bool = smart_db
        .fetch_scalar(&IsEnrolledQueryView::new(user_id, formation_id))
        .await
        .map_err(|_| GetMeFormationByIdError::DatabaseError)?;
    if !enrolled {
        return Err(GetMeFormationByIdError::Forbidden);
    }

    let view = GetMyFormationModulesQueryView::new(formation_id, user_id);
    let rows: Vec<FormationModuleRow> = smart_db
        .fetch_all(&view)
        .await
        .map_err(|_| GetMeFormationByIdError::DatabaseError)?;

    let modules = rows.into_iter().map(map_module).collect();

    Ok(GetFormationResponseView { modules })
}

#[utoipa::path(
    get,
    params(FormationIdParams),
    path = "",
    summary = "List the modules of one of your formations",
    description = "Returns the modules of a formation with, for each one, the `completed` flag \
                   of the user carried by the JWT.\n\n\
                   The progress returned is always the caller's. To read another agent's \
                   progress, use `GET /api/v1/admin/users/{user_id}/{formation_id}`.\n\n\
                   Only available to a caller enrolled in the formation: `403` otherwise, admins \
                   included (an admin enrols through \
                   `POST /api/v1/admin/formations/{formation_id}/`). An unknown formation also \
                   answers `403`, not `404`, so the route never reveals which formation ids \
                   exist (same rule as the module routes). A formation without modules returns \
                   an empty `modules` list.",
    responses(
        (
            status = 200,
            description = "Modules of the formation and their completion state for the caller.",
            body = GetFormationResponseView,
            example = json!({
                "modules": [
                    { "id": 11, "name": "Les principes du RGPD", "description": "Licéité, minimisation, durée de conservation", "completed": true },
                    { "id": 12, "name": "Gérer une demande d'accès", "description": "Procédure et délais", "completed": false }
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
            description = "The caller is not enrolled in this formation, or the formation does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
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
pub async fn get_my_formation_by_id(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<FormationIdParams>,
) -> Result<impl Responder, GetMeFormationByIdError> {
    let params = params.into_inner();
    let formation =
        trigger_get_my_formation_by_id(state, params.formation_id, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(formation))
}
