use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::admin::formations::create_course::view::CreateCourseQueryView;
use crate::database::admin::formations::save_file::view::SaveFileQueryView;
use crate::database::admin::formations::save_module::view::SaveModuleQueryView;
use crate::endpoints::v1::admin::formations::post::view::{CourseBody, CourseIdView};
use crate::endpoints::validation::{ValidatedJson, ValidationError};
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateFormationError {
    BadRequest(String),
    DatabaseError,
}

impl std::fmt::Display for CreateFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateFormationError::BadRequest(reason) => write!(f, "{reason}"),
            CreateFormationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for CreateFormationError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateFormationError::BadRequest(_) => StatusCode::BAD_REQUEST,
            CreateFormationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Position (from 1) of the item at `index` of a list of at most `MAX_ITEMS`.
fn position(index: usize) -> i32 {
    i32::try_from(index + 1).unwrap_or(i32::MAX)
}

/// Saves the modules of `body` and their files in `formation_id`, in the order of the request:
/// a module or file with an `id` is updated (a file moved to its new module), the others are
/// inserted. Shared by the creation and the replacement of a formation, inside their transaction.
///
/// # Errors
///
/// The error of the first query that fails; the caller drops the transaction.
pub async fn save_modules(
    tx: &mut SmartTransaction,
    formation_id: u64,
    body: &CourseBody,
) -> Result<(), ApiLibError> {
    for (m, module) in body.modules.iter().enumerate() {
        let module_id: i32 = tx
            .fetch_scalar(&SaveModuleQueryView::new(
                formation_id,
                module.id,
                &module.fields(position(m)),
            ))
            .await?;
        for (f, file) in module.files.iter().enumerate() {
            let _: i32 = tx
                .fetch_scalar(&SaveFileQueryView::new(
                    id_from_sql(module_id),
                    file.id,
                    &file.fields(position(f)),
                ))
                .await?;
        }
    }
    Ok(())
}

async fn trigger_create_formation(
    state: web::Data<AppState>,
    body: CourseBody,
) -> Result<CourseIdView, CreateFormationError> {
    if let Some(field) = body.first_id_field() {
        return Err(CreateFormationError::BadRequest(
            ValidationError::new(&field, "a new formation has no existing module or content")
                .to_string(),
        ));
    }
    let db_error = || {
        log_error(
            "trigger_create_formation",
            CreateFormationError::DatabaseError,
        )
    };

    // The course, its modules and their files are written together or not at all.
    let mut tx = state.get_smart_db().begin().await.map_err(db_error())?;
    let formation_id: i32 = tx
        .fetch_scalar(&CreateCourseQueryView::new(&body.fields()))
        .await
        .map_err(db_error())?;
    let formation_id = id_from_sql(formation_id);
    save_modules(&mut tx, formation_id, &body)
        .await
        .map_err(db_error())?;
    tx.commit().await.map_err(db_error())?;
    Ok(CourseIdView { id: formation_id })
}

#[utoipa::path(
    post,
    path = "",
    summary = "Create a formation",
    description = "Creates a formation with its modules and their contents, in one transaction: \
                   the order of the arrays gives the display order. Nobody is enrolled: see \
                   `POST /api/v1/admin/formations/{formation_id}/`.\n\n\
                   Without file upload, a content can only be a `link` (its `url`, an absolute \
                   `https://` URL, is required), a `quiz` or `other`; any other type answers \
                   `400`, like an `id` in the body (ids only serve \
                   `PATCH /api/v1/admin/formations/{formation_id}/`). Every field is validated \
                   before anything is written: a `400` names the first invalid field (e.g. \
                   `modules[0].files[1].url`).\n\n\
                   Admin only: a caller without the Admin role gets `403`.",
    request_body(
        content = CourseBody,
        description = "The formation, its modules and their contents.",
        example = json!({
            "name": "GDPR for local government staff",
            "description": "Obligations and good practices",
            "category": "Data protection",
            "level": "beginner",
            "is_mandatory": true,
            "deadline": "2026-12-31",
            "source": "internal",
            "instructor": "Data protection officer",
            "modules": [
                {
                    "name": "The principles of the GDPR",
                    "description": "Lawfulness, minimisation, retention period",
                    "files": [
                        { "title": "Regulation text", "file_type": "link", "url": "https://eur-lex.europa.eu/eli/reg/2016/679/oj", "duration_seconds": 600 },
                        { "title": "Check your knowledge", "file_type": "quiz", "duration_seconds": 300, "is_required": false }
                    ]
                }
            ]
        })
    ),
    responses(
        (
            status = 201,
            description = "Formation created.",
            body = CourseIdView,
            example = json!({ "id": 12 })
        ),
        (
            status = 400,
            description = "Malformed body or invalid field (named in the answer): empty or too long text, unknown `level` / `source` / `file_type`, invalid `deadline`, a new content of a stored-file type, a `link` without an absolute `https://` URL (`javascript:`, `data:`, `http:` and relative URLs are refused), a `url` on another type, or an `id`.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `modules[0].files[1].url`: must be an absolute https:// URL with a host")
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
#[post("/")]
pub async fn create_formation(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    body: ValidatedJson<CourseBody>,
) -> Result<impl Responder, CreateFormationError> {
    let created = trigger_create_formation(state, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(created))
}
