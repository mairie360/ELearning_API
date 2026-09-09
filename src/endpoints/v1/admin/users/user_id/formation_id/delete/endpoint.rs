use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::users::unsub_user_formation::view::UnsubUserFormationQueryView;
use crate::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use crate::endpoints::v1::admin::users::user_id::formation_id::AdminUserFormationIdParams;

#[derive(Debug, Clone, PartialEq)]
pub enum UnsubFormationError {
    DatabaseError,
    UnknownFormations,
}

impl std::fmt::Display for UnsubFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnsubFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            UnsubFormationError::UnknownFormations => {
                write!(f, "Unknown formations.")
            }
        }
    }
}

impl ResponseError for UnsubFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            UnsubFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            UnsubFormationError::UnknownFormations => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_unsub_formation(
    state: web::Data<AppState>,
    formation_id: u64,
    user_id: u64,
) -> Result<(), UnsubFormationError> {
    let smart_db = state.get_smart_db();

    let exists_view = DoesCourseExistQueryView::new(formation_id);
    let exists: bool = smart_db
        .fetch_scalar(&exists_view)
        .await
        .map_err(|_| UnsubFormationError::DatabaseError)?;
    if !exists {
        return Err(UnsubFormationError::UnknownFormations);
    }

    let view = UnsubUserFormationQueryView::new(user_id, formation_id);
    smart_db
        .execute(view)
        .await
        .map_err(|_| UnsubFormationError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "Formation unsubscribed successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin - Users",
    params(
        AdminUserFormationIdParams
    ),
    security(
        ("jwt" = [])
    )
)]
#[delete("/")]
pub async fn unsub_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminUserFormationIdParams>,
) -> Result<impl Responder, UnsubFormationError> {
    trigger_unsub_formation(state, params.formation_id, params.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
