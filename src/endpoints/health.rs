use actix_web::{get, HttpResponse, Responder};
use utoipa::OpenApi;

/// Liveness probe: the process accepts connections. See `ready` for the
/// readiness probe, which checks the dependencies.
#[utoipa::path(
    get,
    path = "health",
    summary = "Liveness probe",
    description = "Answers `OK` as soon as the process accepts connections. Unauthenticated \
                   route, used as the liveness probe by Kubernetes. It checks neither the \
                   database nor Redis, so that an outage of a dependency does not get the pod \
                   restarted: `GET /ready` is the probe that checks them.",
    responses(
        (
            status = 200,
            description = "The service accepts connections.",
            body = String,
            content_type = "text/plain",
            example = json!("OK")
        )
    ),
    tag = "Service"
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(OpenApi)]
#[openapi(paths(health,))]
pub struct HealthDoc;
