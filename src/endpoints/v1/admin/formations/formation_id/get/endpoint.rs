use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
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
use crate::error::log_error;

fn map_content(row: AdminModuleContentRow) -> AdminModuleContent {
    AdminModuleContent::new(id_from_sql(row.id()), row.file_name(), row.file_type())
}

fn map_module(row: AdminFormationModuleRow) -> AdminFormationModule {
    AdminFormationModule::new(
        id_from_sql(row.id()),
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
        .map_err(log_error(
            "trigger_get_formation_by_id",
            GetFormationByIdError::DatabaseError,
        ))?;
    if !exists {
        return Err(GetFormationByIdError::NotFound);
    }

    let view = GetFormationModulesQueryView::new(formation_id, details);
    let rows: Vec<AdminFormationModuleRow> = smart_db.fetch_all(&view).await.map_err(log_error(
        "trigger_get_formation_by_id",
        GetFormationByIdError::DatabaseError,
    ))?;

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
    summary = "List the modules of a catalogue formation",
    description = "Returns the modules of a formation, regardless of anyone's progress: it is \
                   the structure of the course, not its follow-up. For the progress of a given \
                   agent, see `GET /api/v1/admin/users/{user_id}/{formation_id}/`.\n\n\
                   `details=true` makes the answer go down to the attachments of each \
                   module. Without it, `content` is `null`, not an empty array.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Modules de la formation.",
            body = GetFormationByIdResultView,
            example = json!({
                "modules": [
                    {
                        "id": 11,
                        "name": "The principles of the GDPR",
                        "description": "Lawfulness, minimisation, retention period",
                        "content": [
                            { "id": 31, "file_name": "rgpd-principes.pdf", "file_type": "pdf" }
                        ]
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or the JSON body is malformed.",
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
            status = 429,
            description = "The caller exceeded their request quota (per user, `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST`). `Retry-After` gives the seconds to wait.",
            body = String,
            content_type = "text/plain",
            example = json!("Too many requests, retry in 1 s.")
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
            description = "No formation has this id.",
            body = String,
            content_type = "text/plain",
            example = json!("The formation was not found.")
        ),
        (
            status = 500,
            description = "Database error.",
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
