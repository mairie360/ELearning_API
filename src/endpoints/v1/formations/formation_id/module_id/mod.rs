pub mod complete;
pub mod doc;
pub mod get;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ModuleIdParams {
    formation_id: u64,
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
            .service(complete::endpoint::complete_module),
    );
}
