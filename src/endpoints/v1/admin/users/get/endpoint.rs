use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use crate::endpoints::v1::admin::users::get::view::{GetUsersResultView, User};

fn map_user(row: UserRow) -> User {
    User {
        id: row.id() as u64,
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
) -> Result<GetUsersResultView, GetUsersError> {
    let view = GetUsersQueryView::new();
    let rows: Vec<UserRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetUsersError::DatabaseError)?;

    let users = rows.into_iter().map(map_user).collect();

    Ok(GetUsersResultView { users })
}

#[utoipa::path(
    get,
    path = "",
    summary = "Lister les agents inscrits à au moins une formation",
    description = "Renvoie les utilisateurs suivis par ce module, c'est-à-dire ceux qui ont au \
                   moins une inscription. Ce n'est **pas** l'annuaire complet de la plateforme : \
                   pour celui-ci, voir `GET /api/v1/user/` de Core API.\n\n\
                   Vue de liste : la progression n'est pas incluse, il faut passer par \
                   `GET /api/v1/admin/users/{user_id}/`.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    responses(
        (
            status = 200,
            description = "Agents ayant au moins une inscription.",
            body = GetUsersResultView,
            example = json!({
                "users": [
                    { "id": 42, "name": "Jean Dupont" },
                    { "id": 51, "name": "Amina Bensaïd" }
                ]
            })
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
pub async fn get_users(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
) -> Result<impl Responder, GetUsersError> {
    let formations = trigger_get_users(state).await?;
    Ok(HttpResponse::Ok().json(formations))
}
