pub mod access;
pub mod attachment_id;
pub mod complete;
pub mod doc;
pub mod get;

/// Path parameters of the routes of a formation module.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ModuleIdParams {
    /// Id of the formation.
    #[param(example = 4)]
    formation_id: u64,
    /// Id of the module, in the formation of the path.
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
