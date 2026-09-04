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
    responses(
        (status = 200, description = "Users with formations retrieved successfully", body = GetUserFormation),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Module not found"),
        (status = 500, description = "Internal server error")
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
