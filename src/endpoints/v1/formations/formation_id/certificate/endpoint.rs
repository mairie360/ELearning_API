use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::certificate::{render, Attestation};
use crate::database::formations::get_certificate::view::{CertificateRow, GetCertificateQueryView};
use crate::endpoints::v1::formations::formation_id::FormationIdParams;
use crate::endpoints::v1::formations::{pdf_attachment, PdfFile};
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum GetCertificateError {
    Forbidden,
    NotCompleted,
    DatabaseError,
}

impl std::fmt::Display for GetCertificateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCertificateError::Forbidden => {
                write!(f, "You are not enrolled in this formation.")
            }
            GetCertificateError::NotCompleted => {
                write!(f, "The formation is not completed yet.")
            }
            GetCertificateError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetCertificateError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetCertificateError::Forbidden => StatusCode::FORBIDDEN,
            GetCertificateError::NotCompleted => StatusCode::CONFLICT,
            GetCertificateError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// The attestation of a completed formation, `None` while it is not completed.
fn attestation(row: &CertificateRow) -> Option<Attestation> {
    Some(Attestation {
        first_name: row.first_name().to_string(),
        last_name: row.last_name().to_string(),
        course_title: row.title()?.to_string(),
        category: row.category().map(str::to_string),
        completed_at: row.completed_at()?,
        duration_seconds: row.duration_seconds(),
    })
}

async fn trigger_get_certificate(
    state: web::Data<AppState>,
    user_id: u64,
    formation_id: u64,
) -> Result<Vec<u8>, GetCertificateError> {
    let view = GetCertificateQueryView::new(user_id, formation_id);
    let rows: Vec<CertificateRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_certificate",
                GetCertificateError::DatabaseError,
            ))?;
    // Same rule as the other routes of a formation: an unknown formation reads as one the caller
    // is not enrolled in.
    let row = rows
        .into_iter()
        .next()
        .filter(CertificateRow::enrolled)
        .ok_or(GetCertificateError::Forbidden)?;
    if !row.completed() {
        return Err(GetCertificateError::NotCompleted);
    }
    let attestation = attestation(&row).ok_or(GetCertificateError::NotCompleted)?;
    Ok(render(&[attestation]))
}

#[utoipa::path(
    get,
    params(FormationIdParams),
    path = "",
    summary = "Download the attestation of a completed formation",
    description = "Returns a one-page PDF attesting that the caller completed the formation \
                   (written in French: the agent's first and last name, the title of the \
                   formation, its completion date as dd/mm/yyyy, the training duration computed \
                   from the durations of its contents, and its category when it has one). It is \
                   served as a download named `attestation-<formation_id>.pdf`.\n\n\
                   Only a caller enrolled in the formation (`403` otherwise, admins included, and \
                   for an unknown formation) who completed it (`409` otherwise).",
    responses(
        (
            status = 200,
            description = "The attestation, as a PDF download (`Content-Disposition: attachment`).",
            content_type = "application/pdf",
            body = inline(PdfFile),
            headers(
                ("Content-Disposition" = String, description = "`attachment; filename=\"attestation-<formation_id>.pdf\"`")
            )
        ),
        (
            status = 400,
            description = "A path segment is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
            description = "The caller is not enrolled in this formation, or the formation does not exist.",
            body = String,
            content_type = "text/plain",
            example = json!("You are not enrolled in this formation.")
        ),
        (
            status = 409,
            description = "The caller has not completed the formation yet.",
            body = String,
            content_type = "text/plain",
            example = json!("The formation is not completed yet.")
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
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Formations",
)]
#[get("/certificate/")]
pub async fn get_certificate(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<FormationIdParams>,
) -> Result<impl Responder, GetCertificateError> {
    let formation_id = params.formation_id();
    let pdf = trigger_get_certificate(state, auth_user.id, formation_id).await?;
    Ok(pdf_attachment(
        pdf,
        &format!("attestation-{formation_id}.pdf"),
    ))
}
