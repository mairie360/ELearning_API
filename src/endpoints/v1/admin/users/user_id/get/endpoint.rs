use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_user_formations::view::{
    GetUserFormationsQueryView, UserFormationModuleRow, UserFormationRow, UserModuleContentRow,
};
use crate::endpoints::v1::admin::users::user_id::get::view::GetUserByIdResultView;
use crate::endpoints::v1::admin::users::user_id::AdminUserIdParams;
use crate::endpoints::v1::admin::users::{
    ProgressStatus, UsersFormation, UsersFormationModule, UsersModuleContent,
};
use crate::endpoints::v1::admin::AdminUserDetailsQuery;
use crate::error::log_error;

fn map_content(row: UserModuleContentRow) -> UsersModuleContent {
    // `course_attachments` has no per-user tracking, so files never carry a
    // `finished_at`; only whole modules do (`user_modules.completed_at`).
    UsersModuleContent::new(
        id_from_sql(row.id()),
        row.file_name(),
        row.file_type(),
        None,
    )
}

fn map_module(row: UserFormationModuleRow) -> UsersFormationModule {
    UsersFormationModule::new(
        id_from_sql(row.id()),
        row.name(),
        row.description().unwrap_or_default(),
        row.content().iter().cloned().map(map_content).collect(),
        row.is_completed(),
        row.completed_at().map(|dt| dt.and_utc()),
    )
}

fn map_formation(row: UserFormationRow) -> UsersFormation {
    UsersFormation::new(
        id_from_sql(row.id()),
        row.name(),
        row.description().unwrap_or_default(),
        row.modules()
            .map(|modules| modules.iter().cloned().map(map_module).collect()),
        row.started_at().map(|dt| dt.and_utc()),
        row.completed_at().map(|dt| dt.and_utc()),
        ProgressStatus::from(row.progress_status().to_string()),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetUserFormationsError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetUserFormationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUserFormationsError::BadRequest => {
                write!(f, "Bad request")
            }
            GetUserFormationsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUserFormationsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUserFormationsError::BadRequest => StatusCode::BAD_REQUEST,
            GetUserFormationsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_user(
    state: web::Data<AppState>,
    user_id: u64,
    details: bool,
) -> Result<GetUserByIdResultView, GetUserFormationsError> {
    let view = GetUserFormationsQueryView::new(user_id, details);
    let rows: Vec<UserFormationRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_user",
                GetUserFormationsError::DatabaseError,
            ))?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetUserByIdResultView { formations })
}

#[utoipa::path(
    get,
    params(AdminUserDetailsQuery, AdminUserIdParams),
    path = "",
    summary = "Read the progress of an agent",
    description = "Returns the formations an agent is enrolled in, with their start and end \
                   dates and progress status. Same as `GET /api/v1/formations/`, but for \
                   any agent, not for the caller.\n\n\
                   `started_at` stays `null` until the agent completes a module: the date \
                   is set by a database trigger on the first completed module, not on \
                   enrolment. `completed_at` stays `null` until the formation is fully \
                   completed.\n\n\
                   `details=true` makes the answer go down to the modules and their \
                   attachments; without it, `modules` is `null`.\n\n\
                   An unknown agent, or one without enrolment, gets an empty list, not an error.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Formations of the agent and their progress on each.",
            body = GetUserByIdResultView,
            example = json!({
                "formations": [
                    {
                        "id": 4,
                        "name": "GDPR for local government staff",
                        "description": "Obligations and good practices",
                        "modules": null,
                        "started_at": "2026-09-02T08:30:00Z",
                        "completed_at": null,
                        "progress_status": "InProgress"
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
            status = 500,
            description = "Database error.",
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
pub async fn get_user_formations(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    query: web::Query<AdminUserDetailsQuery>,
    path_params: web::Path<AdminUserIdParams>,
) -> Result<impl Responder, GetUserFormationsError> {
    let formations = trigger_get_user(state, path_params.user_id, query.details()).await?;
    Ok(HttpResponse::Ok().json(formations))
}
