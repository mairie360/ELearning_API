//! Readiness probe: unlike `/health` (liveness), it answers `200` only when
//! the dependencies of the service answer (MAIR-423).

use std::time::Duration;

use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, ResponseError};
use mairie360_api_lib::state::AppState;
use utoipa::OpenApi;

use crate::database::ping::view::PingQueryView;

/// How long each dependency gets to answer before it is reported down: a probe
/// must answer before the kubelet's own timeout (1 s by default).
pub const DEPENDENCY_TIMEOUT: Duration = Duration::from_millis(800);

/// The dependency that did not answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessError {
    Postgres,
    Redis,
}

impl std::fmt::Display for ReadinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadinessError::Postgres => write!(f, "PostgreSQL is unreachable."),
            ReadinessError::Redis => write!(f, "Redis is unreachable."),
        }
    }
}

impl ResponseError for ReadinessError {
    fn status_code(&self) -> StatusCode {
        StatusCode::SERVICE_UNAVAILABLE
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Checks that Postgres answers `SELECT 1`.
pub async fn check_postgres(state: &AppState) -> Result<(), ReadinessError> {
    let view = PingQueryView::new();
    let ping = state.get_smart_db().fetch_scalar::<i32, _>(&view);
    match tokio::time::timeout(DEPENDENCY_TIMEOUT, ping).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => {
            log::warn!(target: "readiness", "PostgreSQL ping failed: {err}");
            Err(ReadinessError::Postgres)
        }
        Err(_) => {
            log::warn!(target: "readiness", "PostgreSQL ping timed out");
            Err(ReadinessError::Postgres)
        }
    }
}

/// Checks that Redis answers a command (`EXISTS` on a key that is never set).
pub async fn check_redis(state: &AppState) -> Result<(), ReadinessError> {
    let ping = state.get_redis().key_exist("readiness-probe");
    match tokio::time::timeout(DEPENDENCY_TIMEOUT, ping).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => {
            log::warn!(target: "readiness", "Redis ping failed: {err}");
            Err(ReadinessError::Redis)
        }
        Err(_) => {
            log::warn!(target: "readiness", "Redis ping timed out");
            Err(ReadinessError::Redis)
        }
    }
}

#[utoipa::path(
    get,
    path = "ready",
    summary = "Readiness probe",
    description = "Answers `200 OK` when PostgreSQL answers `SELECT 1` and Redis answers a \
                   command, each within 800 ms; `503` otherwise, naming the first dependency \
                   that failed. Unauthenticated route, used as the readiness probe by \
                   Kubernetes (a `503` takes the pod out of the service without restarting it) \
                   and by the Docker test stacks.\n\n\
                   `GET /health` is the liveness probe: it only tells that the process accepts \
                   connections.",
    responses(
        (
            status = 200,
            description = "PostgreSQL and Redis answer.",
            body = String,
            content_type = "text/plain",
            example = json!("OK")
        ),
        (
            status = 503,
            description = "PostgreSQL (checked first) or Redis did not answer in time.",
            body = String,
            content_type = "text/plain",
            example = json!("PostgreSQL is unreachable.")
        )
    ),
    tag = "Service"
)]
#[get("/ready")]
pub async fn ready(state: web::Data<AppState>) -> Result<HttpResponse, ReadinessError> {
    check_postgres(&state).await?;
    check_redis(&state).await?;
    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(OpenApi)]
#[openapi(paths(ready,))]
pub struct ReadyDoc;
