use crate::endpoints::health::HealthDoc;
use crate::endpoints::hello::HelloDoc;
use crate::endpoints::v1::doc::V1Doc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

// Dans votre ApiDoc principale
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ELearning API — Mairie 360",
        version = "1.0.0",
        description = "\
API de formation en ligne de la plateforme **Mairie 360** : catalogue de formations, modules, \
pièces jointes et suivi de progression des agents. Les comptes et les rôles vivent dans Core API.

## Vocabulaire

Une **formation** regroupe des **modules**, chacun portant des **pièces jointes** (vidéo ou PDF). \
Un agent inscrit à une formation progresse module par module ; la formation passe de \
`NotStarted` à `InProgress` puis `Completed` au fil des modules qu'il termine.

## Deux familles de routes

Les routes `/api/v1/formations/…` sont la vue **de l'agent connecté** : elles ne montrent que ses \
propres inscriptions et sa propre progression, déduites du JWT.

Les routes `/api/v1/admin/…` sont la vue **de gestion** : catalogue complet, inscription et \
désinscription d'un agent, consultation de la progression de n'importe qui.

Attention : contrairement à Core API, le préfixe `/admin` n'est **pas** protégé par un contrôle de \
rôle. Tout utilisateur authentifié peut appeler ces routes ; elles ne renvoient donc jamais `403`.

## Pièces jointes

Les fichiers ne transitent pas par l'API. \
`GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/` renvoie une URL signée à \
durée de vie limitée, que le client utilise ensuite directement. Un `502` sur cette route signale \
que le stockage de fichiers est injoignable, pas que la pièce jointe est absente.

## Error format

Error responses (`4xx` and `5xx`) have a **`text/plain`** body holding the error message, not a \
JSON object. Every response carries `X-Content-Type-Options: nosniff`.

Statuses returned across the API, before the handler runs:

| Status | Meaning |
| --- | --- |
| `400` | URL segment that is not an integer, or malformed JSON body. |
| `401` | `Authorization` header missing or malformed, invalid or expired JWT, or revoked session. |
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
        (name = "Service", description = "Sondes techniques non authentifiées, utilisées par Docker et Kubernetes.")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = HelloDoc),
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
