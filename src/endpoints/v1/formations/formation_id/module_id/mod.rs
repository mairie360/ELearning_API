pub mod access;
pub mod attachment_id;
pub mod complete;
pub mod doc;
pub mod get;

/// Paramètres de chemin des routes d'un module de formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ModuleIdParams {
    /// Identifiant de la formation.
    #[param(example = 4)]
    formation_id: u64,
    /// Identifiant du module, dans la formation du chemin.
    #[param(example = 11)]
    module_id: u64,
}

impl ModuleIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }

    pub fn module_id(&self) -> u64 {
        self.module_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{module_id}")
            .service(get::endpoint::get_module)
            .service(complete::endpoint::complete_module)
            .configure(attachment_id::config),
    );
}
