use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::get_formations::view::{
    AdminFormationModuleRow, AdminFormationRow, AdminModuleContentRow, GetFormationsQueryView,
};
use crate::endpoints::v1::admin::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::admin::formations::{
    AdminFormation, AdminFormationModule, AdminModuleContent,
};
use crate::endpoints::v1::admin::{AdminPageQuery, AdminUserDetailsQuery};
use crate::logging::log_error;

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

fn map_formation(row: AdminFormationRow) -> AdminFormation {
    AdminFormation::new(
        id_from_sql(row.id()),
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
    page: &AdminPageQuery,
) -> Result<GetFormationsResultView, GetFormationsError> {
    let view = GetFormationsQueryView::new(details, page.limit(), page.offset());
    let rows: Vec<AdminFormationRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_formations",
                GetFormationsError::DatabaseError,
            ))?;

    let formations = rows.into_iter().map(map_formation).collect();

    Ok(GetFormationsResultView { formations })
}

#[utoipa::path(
    get,
    params(
        AdminUserDetailsQuery,
        AdminPageQuery,
    ),
    path = "",
    summary = "List the formation catalogue",
    description = "Returns one page of the formations of the platform, in `id` order, whatever \
                   the enrolments of the caller — unlike `GET /api/v1/formations/`, which only \
                   shows the caller's own.\n\n\
                   Paginated: `limit` (default 50, capped to 200) and `offset` (default 0). A \
                   page shorter than `limit` is the last one; an `offset` past the end returns \
                   an empty list.\n\n\
                   `details=true` expands the response down to the modules and their \
                   attachments in a single query. Without it, `modules` is `null`, not an empty \
                   array: the information was not requested, it is not a formation without \
                   modules.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "One page of the catalogue, possibly empty.",
            body = GetFormationsResultView,
            example = json!({
                "formations": [
                    { "id": 4, "name": "RGPD pour les agents territoriaux", "description": "Obligations et bonnes pratiques", "modules": null },
                    { "id": 9, "name": "Accueil du public en situation de handicap", "description": "", "modules": null }
                ]
            })
        ),
        (
            status = 400,
            description = "`details` is not a boolean, or `limit` / `offset` is not a non-negative integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Query deserialize error: invalid type: string \"oui\", expected a boolean")
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
    page: web::Query<AdminPageQuery>,
) -> Result<impl Responder, GetFormationsError> {
    let formations = trigger_get_formations(state, query.details(), &page).await?;
    Ok(HttpResponse::Ok().json(formations))
}
