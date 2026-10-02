use crate::endpoints::health::HealthDoc;
use crate::endpoints::ready::ReadyDoc;
use crate::endpoints::v1::doc::V1Doc;
use actix_web::web;
use mairie360_api_lib::env_manager::get_env_var;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// Environment variable that mounts Swagger UI and the served spec.
pub const SWAGGER_ENABLED_ENV: &str = "SWAGGER_ENABLED";

/// Whether `SWAGGER_ENABLED` is `true` (or `1`). Off by default, so a
/// deployment that does not set it (production) serves neither Swagger UI nor
/// `/api-docs/openapi.json` (MAIR-424). The dev, ZAP and k6 stacks set it; the
/// published contract package is built from `cargo open_api`, not from the
/// served spec.
pub fn is_enabled() -> bool {
    get_env_var(SWAGGER_ENABLED_ENV).is_some_and(|value| {
        let value = value.trim();
        value == "1" || value.eq_ignore_ascii_case("true")
    })
}

/// Mounts Swagger UI at `/swagger-ui/` and the spec at
/// `/api-docs/openapi.json`, outside of any authentication, when `enabled`.
pub fn config(cfg: &mut web::ServiceConfig, enabled: bool) {
    if enabled {
        cfg.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ELearning API — Mairie 360",
        version = "1.0.0",
        description = "\
E-learning API of the **Mairie 360** platform: course catalogue, modules, attachments and agent \
progress tracking. Accounts and roles live in Core API.

## Vocabulary

A **formation** (course) groups **modules**, each carrying **attachments** (video or PDF). An \
agent enrolled in a formation progresses module by module; the formation goes from \
`NotStarted` to `InProgress` then `Completed` as the agent completes its modules.

## Two families of routes

The `/api/v1/formations/…` routes are the view **of the signed-in agent**: they only show the \
agent's own enrolments and progress, derived from the JWT. The routes of a formation \
(`/api/v1/formations/{formation_id}/…`) answer `403` when the caller is not enrolled in it, \
admins included. An unknown formation answers `403` too, so these routes never reveal which \
formation ids exist. The routes of a module answer `404` when the module does not belong to the \
formation.

The `/api/v1/admin/…` routes are the **management** view: full catalogue, enrolling and \
unenrolling an agent, reading anyone's progress. Like in Core API, they are restricted to admins: \
a non-admin caller gets `403` before the handler runs.

## Attachments

Files never go through the API. \
`GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/` returns a short-lived signed \
URL that the client then uses directly. A `502` on that route means the file storage is \
unreachable, not that the attachment is missing.

## Error format

Error responses (`4xx` and `5xx`) have a **`text/plain`** body holding the error message, not a \
JSON object. Every response carries `X-Content-Type-Options: nosniff`.

Statuses returned across the API, before the handler runs:

| Status | Meaning |
| --- | --- |
| `400` | A path segment is not an integer, or the JSON body is malformed. |
| `401` | `Authorization` header missing or malformed, invalid or expired JWT, or revoked session. |
| `403` | `/api/v1/admin/…` only: the caller is not an admin. |
| `500` | Database or Redis failure. |
",
        contact(
            name = "Équipe Mairie 360",
            url = "https://github.com/mairie360"
        ),
        license(
            name = "Propriétaire",
            identifier = "LicenseRef-mairie360-proprietary"
        )
    ),
    servers(
        (url = "http://localhost:3006", description = "Développement local (cargo run)"),
        (url = "http://development.mairie360.fr", description = "Pile Docker de développement (nginx)")
    ),
    tags(
        (name = "Formations", description = "Vue de l'agent connecté : ses formations, ses modules, sa progression et les URL de ses pièces jointes."),
        (name = "Admin - Formations", description = "Catalogue complet des formations et inscription des agents."),
        (name = "Admin - Users", description = "Progression des agents et désinscription."),
        (name = "Service", description = "Unauthenticated technical probes used by Docker and Kubernetes: `/health` (liveness) and `/ready` (readiness).")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = ReadyDoc),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Sans ce modifier, les opérations qui déclarent `security(("jwt" = []))` référencent un schéma
/// absent du contrat : Swagger UI n'offre pas de bouton « Authorize » et les clients générés
/// pointent dans le vide.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT émis par Core API (`POST /api/v1/auth/login`)."))
                    .build(),
            ),
        )
    }
}
