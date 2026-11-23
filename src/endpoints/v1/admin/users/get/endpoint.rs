use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use crate::endpoints::v1::admin::users::get::view::{
    AdminUsersFilterQuery, GetUsersResultView, User,
};
use crate::endpoints::v1::admin::AdminPageQuery;
use crate::endpoints::validation::{Validate, ValidationError};
use crate::error::log_error;

fn map_user(row: UserRow) -> User {
    User {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
        first_name: row.first_name().to_string(),
        last_name: row.last_name().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetUsersError {
    InvalidFilter(ValidationError),
    DatabaseError,
}

impl std::fmt::Display for GetUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUsersError::InvalidFilter(error) => write!(f, "{error}"),
            GetUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUsersError::InvalidFilter(_) => StatusCode::BAD_REQUEST,
            GetUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_users(
    state: web::Data<AppState>,
    page: &AdminPageQuery,
    filter: &AdminUsersFilterQuery,
) -> Result<GetUsersResultView, GetUsersError> {
    filter.validate().map_err(GetUsersError::InvalidFilter)?;
    let view = GetUsersQueryView::filtered(
        page.limit(),
        page.offset(),
        filter.search(),
        filter.exclude_formation_id(),
    );
    let rows: Vec<UserRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(log_error("trigger_get_users", GetUsersError::DatabaseError))?;

    let users = rows.into_iter().map(map_user).collect();

    Ok(GetUsersResultView { users })
}

#[utoipa::path(
    get,
    path = "",
    params(
        AdminPageQuery,
        AdminUsersFilterQuery,
    ),
    summary = "List the active users",
    description = "Returns one page of the active (non-archived) users of the platform, in \
                   `id` order, to pick who to enrol in a formation.\n\n\
                   Optional filters (MAIR-817): `search` keeps the users whose `first last` \
                   name, `last first` name or e-mail contains the term, whatever the letter \
                   case; `exclude_formation_id` leaves out the users already enrolled in that \
                   formation.\n\n\
                   Paginated: `limit` (default 50, capped to 200) and `offset` (default 0). A \
                   page shorter than `limit` is the last one; an `offset` past the end returns \
                   an empty list.\n\n\
                   List view: progress is not included, see \
                   `GET /api/v1/admin/users/{user_id}/`.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "One page of active users, possibly empty.",
            body = GetUsersResultView,
            example = json!({
                "users": [
                    { "id": 42, "name": "Jean Dupont", "first_name": "Jean", "last_name": "Dupont" },
                    { "id": 51, "name": "Amina Bensaïd", "first_name": "Amina", "last_name": "Bensaïd" }
                ]
            })
        ),
        (
            status = 400,
            description = "`limit`, `offset` or `exclude_formation_id` is not a non-negative integer, `exclude_formation_id` is outside 1 to 2147483647, or `search` is blank once trimmed, longer than 100 characters or holds a control character.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `search`: must not be empty")
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
pub async fn get_users(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    page: web::Query<AdminPageQuery>,
    filter: web::Query<AdminUsersFilterQuery>,
) -> Result<impl Responder, GetUsersError> {
    let users = trigger_get_users(state, &page, &filter).await?;
    Ok(HttpResponse::Ok().json(users))
}
