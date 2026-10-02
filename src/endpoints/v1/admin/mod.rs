pub mod doc;
pub mod formations;
pub mod users;

use mairie360_api_lib::security::AdminMiddleware;

/// Page size of the paginated admin lists when `limit` is absent.
pub const DEFAULT_PAGE_SIZE: u32 = 50;
/// Largest page the paginated admin lists return, whatever `limit` asks for.
pub const MAX_PAGE_SIZE: u32 = 200;

/// `limit` / `offset` of the paginated admin lists (`GET /api/v1/admin/users/`,
/// `GET /api/v1/admin/formations/`), MAIR-395 / MAIR-425.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminPageQuery {
    /// Maximum number of items to return. Defaults to 50; values above 200 are capped to 200
    /// and `0` is raised to 1.
    #[param(example = 50, minimum = 1, maximum = 200)]
    limit: Option<u32>,
    /// Number of items to skip, in `id` order. Defaults to 0. Page `n` (from 0) is
    /// `offset = n * limit`; a page shorter than `limit` is the last one.
    #[param(example = 0, minimum = 0)]
    offset: Option<u32>,
}

impl AdminPageQuery {
    /// The page size actually used, within `1..=MAX_PAGE_SIZE`.
    pub fn limit(&self) -> u32 {
        self.limit
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

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
