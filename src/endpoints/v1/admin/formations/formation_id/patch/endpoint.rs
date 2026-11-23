use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_to_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::check_course_content::view::{
    CheckCourseContentQueryView, CourseContentCheckRow, KeptFile,
};
use crate::database::admin::formations::delete_course_content::view::{
    CourseContent, DeleteCourseContentQueryView,
};
use crate::database::admin::formations::update_course::view::UpdateCourseQueryView;
use crate::endpoints::v1::admin::formations::formation_id::AdminFormationWriteParams;
use crate::endpoints::v1::admin::formations::post::endpoint::save_modules;
use crate::endpoints::v1::admin::formations::post::view::{CourseBody, CourseIdView};
use crate::endpoints::validation::{ValidatedJson, ValidationError};
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateFormationError {
    BadRequest(String),
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for UpdateFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateFormationError::BadRequest(reason) => write!(f, "{reason}"),
            UpdateFormationError::NotFound => write!(f, "The formation was not found."),
            UpdateFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for UpdateFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateFormationError::BadRequest(_) => StatusCode::BAD_REQUEST,
            UpdateFormationError::NotFound => StatusCode::NOT_FOUND,
            UpdateFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// The `400` of the first id of `body` the check refused, `None` when every id is consistent.
fn refused_id(body: &CourseBody, check: &CourseContentCheckRow) -> Option<UpdateFormationError> {
    let refused = |field: &str, reason: &str| {
        Some(UpdateFormationError::BadRequest(
            ValidationError::new(field, reason).to_string(),
        ))
    };
    for (m, module) in body.modules.iter().enumerate() {
        if module
            .id
            .is_some_and(|id| check.unknown_modules().contains(&id_to_sql(id)))
        {
            return refused(
                &format!("modules[{m}].id"),
                "is not a module of this formation",
            );
        }
    }
    for (field, id, _) in body.kept_files() {
        let id = id_to_sql(id);
        if check.unknown_files().contains(&id) {
            return refused(&field, "is not a content of this formation");
        }
        if check.retyped_files().contains(&id) {
            return refused(
                &field,
                "the type of a stored file cannot change, nor can a content become one",
            );
        }
    }
    None
}

async fn trigger_update_formation(
    state: web::Data<AppState>,
    formation_id: u64,
    body: CourseBody,
) -> Result<CourseIdView, UpdateFormationError> {
    let db_error = || {
        log_error(
            "trigger_update_formation",
            UpdateFormationError::DatabaseError,
        )
    };

    // One transaction: the update of the course row locks it first, so two replacements of the
    // same formation run one after the other and the id checks cannot go stale (MAIR-420).
    let mut tx = state.get_smart_db().begin().await.map_err(db_error())?;
    let found: bool = tx
        .fetch_scalar(&UpdateCourseQueryView::new(formation_id, &body.fields()))
        .await
        .map_err(db_error())?;
    if !found {
        return Err(UpdateFormationError::NotFound);
    }

    let module_ids = body.module_ids();
    let kept_files: Vec<(String, u64, _)> = body.kept_files();
    let kept = kept_files
        .iter()
        .map(|(_, id, kind)| KeptFile {
            id: id_to_sql(*id),
            file_type: kind.as_db().to_string(),
        })
        .collect::<Vec<_>>();
    let check: CourseContentCheckRow = tx
        .fetch_one(&CheckCourseContentQueryView::new(
            formation_id,
            &module_ids,
            &kept,
        ))
        .await
        .map_err(db_error())?;
    if let Some(error) = refused_id(&body, &check) {
        return Err(error);
    }

    // Files first, modules last: a kept file moved out of a dropped module survives.
    let file_ids: Vec<u64> = kept_files.iter().map(|(_, id, _)| *id).collect();
    tx.execute(&DeleteCourseContentQueryView::new(
        CourseContent::Files,
        formation_id,
        &file_ids,
    ))
    .await
    .map_err(db_error())?;
    save_modules(&mut tx, formation_id, &body)
        .await
        .map_err(db_error())?;
    tx.execute(&DeleteCourseContentQueryView::new(
        CourseContent::Modules,
        formation_id,
        &module_ids,
    ))
    .await
    .map_err(db_error())?;
    tx.commit().await.map_err(db_error())?;
    Ok(CourseIdView { id: formation_id })
}

#[utoipa::path(
    patch,
    params(AdminFormationWriteParams),
    path = "/",
    summary = "Replace a formation",
    description = "Replaces a formation, its modules and their contents with the body (same body \
                   as `POST /api/v1/admin/formations/`), in one transaction.\n\n\
                   A module or content carrying an `id` keeps its row (a content may move to \
                   another module of the formation): the progress of the agents on a kept module \
                   is kept. A kept stored file (video, PDF, document, audio) keeps its file, \
                   only its metadata change, and its type cannot change. The modules and \
                   contents the body does not list are deleted, with the progress on them. An \
                   `id` that is not a module or content of this formation answers `400`.\n\n\
                   The enrolments, statuses and ratings are kept as they are.\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    request_body(
        content = CourseBody,
        description = "The formation as it must become.",
        example = json!({
            "name": "GDPR for local government staff",
            "category": "Data protection",
            "level": "intermediate",
            "is_mandatory": true,
            "deadline": "2027-03-31",
            "modules": [
                {
                    "id": 11,
                    "name": "The principles of the GDPR",
                    "files": [
                        { "id": 27, "title": "Principles of the GDPR", "file_type": "pdf", "duration_seconds": 900 },
                        { "title": "Regulation text", "file_type": "link", "url": "https://eur-lex.europa.eu/eli/reg/2016/679/oj" }
                    ]
                },
                { "name": "Handling an access request" }
            ]
        })
    ),
    responses(
        (
            status = 200,
            description = "Formation replaced.",
            body = CourseIdView,
            example = json!({ "id": 7 })
        ),
        (
            status = 400,
            description = "Malformed body, invalid field (see `POST /api/v1/admin/formations/`), an `id` that is not a module or content of this formation, the type of a stored file changed, or a path segment that is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `modules[0].files[0].id`: is not a content of this formation")
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
            status = 404,
            description = "The formation does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("The formation was not found.")
        ),
        (
            status = 429,
            description = "The caller exceeded their request quota (per user, `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST`). `Retry-After` gives the seconds to wait.",
            body = String,
            content_type = "text/plain",
            example = json!("Too many requests, retry in 1 s.")
        ),
        (
            status = 500,
            description = "Database error. Nothing is written.",
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
#[patch("/")]
pub async fn update_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<AdminFormationWriteParams>,
    body: ValidatedJson<CourseBody>,
) -> Result<impl Responder, UpdateFormationError> {
    let updated = trigger_update_formation(state, params.formation_id(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(updated))
}
