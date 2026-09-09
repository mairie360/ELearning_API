use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::get_formations::view::{
    AdminFormationModuleRow, AdminFormationRow, AdminModuleContentRow, GetFormationsQueryView,
};
use crate::endpoints::v1::admin::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::admin::formations::{
    AdminFormation, AdminFormationModule, AdminModuleContent,
};
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

fn map_formation(row: AdminFormationRow) -> AdminFormation {
    AdminFormation::new(
        row.id() as u64,
        row.name(),
        row.description().unwrap_or_default(),
        row.modules()
            .map(|modules| modules.iter().cloned().map(map_module).collect()),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetFormationsError {
    DatabaseError,
}

impl std::fmt::Display for GetFormationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetFormationsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetFormationsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetFormationsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_formations(
    state: web::Data<AppState>,
    details: bool,
) -> Result<GetFormationsResultView, GetFormationsError> {
    let view = GetFormationsQueryView::new(details);
    let rows: Vec<AdminFormationRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetFormationsError::DatabaseError)?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetFormationsResultView { formations })
}

#[utoipa::path(
    get,
    params(
        AdminUserDetailsQuery,
    ),
    path = "",
    responses(
        (status = 200, description = "Formations retrieved successfully", body = GetFormationsResultView),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Formations",
    security(
        ("jwt" = [])
    )
)]
#[get("/")]
pub async fn get_formations(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    query: web::Query<AdminUserDetailsQuery>,
) -> Result<impl Responder, GetFormationsError> {
    let formations = trigger_get_formations(state, query.details()).await?;
    Ok(HttpResponse::Ok().json(formations))
}
