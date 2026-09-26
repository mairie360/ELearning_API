pub mod doc;
pub mod formations;
pub mod users;

use mairie360_api_lib::security::AdminMiddleware;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminUserDetailsQuery {
    /// `true` fait descendre la réponse d'un niveau supplémentaire (modules, puis pièces
    /// jointes). Par défaut `false`, auquel cas les champs imbriqués valent `null` — ce qui
    /// signifie « non demandé », pas « vide ».
    #[serde(default)]
    #[param(example = true)]
    details: bool,
}

impl AdminUserDetailsQuery {
    pub fn details(&self) -> bool {
        self.details
    }
}

/// Mounts `/admin`. Every route below it is restricted to admins by
/// `AdminMiddleware` (the lib's `is_admin()` check): a valid JWT of a non-admin
/// answers `403 Forbidden: User is not an admin.` before reaching a handler.
pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/admin")
            .wrap(AdminMiddleware)
            .configure(formations::config)
            .configure(users::config),
    );
}
