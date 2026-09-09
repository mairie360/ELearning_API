use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
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

fn map_content(row: UserModuleContentRow) -> UsersModuleContent {
    // `course_attachments` has no per-user tracking, so files never carry a
    // `finished_at`; only whole modules do (`user_modules.completed_at`).
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

fn map_formation(row: UserFormationRow) -> UsersFormation {
    UsersFormation::new(
        row.id() as u64,
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
    let rows: Vec<UserFormationRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetUserFormationsError::DatabaseError)?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetUserByIdResultView { formations })
}

#[utoipa::path(
    get,
    params(AdminUserDetailsQuery, AdminUserIdParams),
    path = "",
    responses(
        (status = 200, description = "User formations retrieved successfully", body = GetUserByIdResultView),
        (status = 500, description = "Internal server error")
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
