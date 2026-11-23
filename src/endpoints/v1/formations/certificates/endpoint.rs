use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::certificate::{render, Attestation};
use crate::database::formations::get_my_certificates::view::{
    CompletedFormationRow, GetMyCertificatesQueryView,
};
use crate::endpoints::v1::formations::{pdf_attachment, PdfFile, YearQuery};
use crate::error::log_error;

#[derive(Debug, Clone, PartialEq)]
pub enum GetCertificatesError {
    InvalidYear,
    NoneCompleted,
    DatabaseError,
}

impl std::fmt::Display for GetCertificatesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCertificatesError::InvalidYear => {
                write!(f, "Invalid `year`: must be between 2000 and 2100")
            }
            GetCertificatesError::NoneCompleted => {
                write!(f, "You completed no formation during this year.")
            }
            GetCertificatesError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetCertificatesError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetCertificatesError::InvalidYear => StatusCode::BAD_REQUEST,
            GetCertificatesError::NoneCompleted => StatusCode::NOT_FOUND,
            GetCertificatesError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

fn attestation(row: CompletedFormationRow) -> Attestation {
    Attestation {
        first_name: row.first_name().to_string(),
        last_name: row.last_name().to_string(),
        course_title: row.title().to_string(),
        category: row.category().map(str::to_string),
        completed_at: row.completed_at(),
        duration_seconds: row.duration_seconds(),
    }
}

async fn trigger_get_certificates(
    state: web::Data<AppState>,
    user_id: u64,
    year: i32,
) -> Result<Vec<u8>, GetCertificatesError> {
    let view = GetMyCertificatesQueryView::new(user_id, year);
    let rows: Vec<CompletedFormationRow> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(log_error(
                "trigger_get_certificates",
                GetCertificatesError::DatabaseError,
            ))?;
    if rows.is_empty() {
        return Err(GetCertificatesError::NoneCompleted);
    }
    let attestations: Vec<Attestation> = rows.into_iter().map(attestation).collect();
    Ok(render(&attestations))
}

#[utoipa::path(
    get,
    params(YearQuery),
    path = "",
    summary = "Download the attestations of a year",
    description = "Returns one PDF holding one attestation page per formation the caller \
                   completed during a calendar year (the current one by default), by completion \
                   date. Each page is the one of \
                   `GET /api/v1/formations/{formation_id}/certificate/`. It is served as a \
                   download named `attestations-<year>.pdf`; `404` when the caller completed no \
                   formation that year.",
    responses(
        (
            status = 200,
            description = "The attestations of the year, as a PDF download (`Content-Disposition: attachment`).",
            content_type = "application/pdf",
            body = inline(PdfFile),
            headers(
                ("Content-Disposition" = String, description = "`attachment; filename=\"attestations-<year>.pdf\"`")
            )
        ),
        (
            status = 400,
            description = "`year` is not an integer between 2000 and 2100.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `year`: must be between 2000 and 2100")
        ),
        (
            status = 401,
            description = "`Authorization` header missing, or JWT invalid or expired.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "The caller completed no formation during the year.",
            body = String,
            content_type = "text/plain",
            example = json!("You completed no formation during this year.")
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
#[get("/certificates/")]
pub async fn get_my_certificates(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    query: web::Query<YearQuery>,
) -> Result<impl Responder, GetCertificatesError> {
    let year = query.year().ok_or(GetCertificatesError::InvalidYear)?;
    let pdf = trigger_get_certificates(state, auth_user.id, year).await?;
    Ok(pdf_attachment(pdf, &format!("attestations-{year}.pdf")))
}
