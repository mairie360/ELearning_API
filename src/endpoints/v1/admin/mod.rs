pub mod doc;
pub mod formations;
pub mod users;

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

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/admin")
            .configure(formations::config)
            .configure(users::config),
    );
}
