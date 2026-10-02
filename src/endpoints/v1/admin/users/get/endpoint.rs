use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use crate::endpoints::v1::admin::users::get::view::{GetUsersResultView, User};
use crate::endpoints::v1::admin::users::AdminUsersPageQuery;
use crate::logging::log_error;

fn map_user(row: UserRow) -> User {
    User {
        id: id_from_sql(row.id()),
        name: row.name().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetUsersError {
    DatabaseError,
}

impl std::fmt::Display for GetUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_users(
    state: web::Data<AppState>,
    page: &AdminUsersPageQuery,
) -> Result<GetUsersResultView, GetUsersError> {
    let view = GetUsersQueryView::new(page.limit(), page.offset());
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
        AdminUsersPageQuery,
    ),
    summary = "List the active users",
    description = "Returns one page of the active (non-archived) users of the platform, in \
                   `id` order, to pick who to enrol in a formation. Whether they are enrolled \
                   in a formation or not does not matter.\n\n\
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
                    { "id": 42, "name": "Jean Dupont" },
                    { "id": 51, "name": "Amina Bensaïd" }
                ]
            })
        ),
        (
            status = 400,
            description = "`limit` or `offset` is not a non-negative integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Query deserialize error: invalid digit found in string")
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
    page: web::Query<AdminUsersPageQuery>,
) -> Result<impl Responder, GetUsersError> {
    let users = trigger_get_users(state, &page).await?;
    Ok(HttpResponse::Ok().json(users))
}
