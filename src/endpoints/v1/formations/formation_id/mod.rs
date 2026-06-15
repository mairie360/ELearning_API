pub mod doc;
pub mod get;
pub mod module_id;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct FormationIdParams {
    formation_id: u64,
}

impl FormationIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{formation_id}")
            .service(get::endpoint::get_my_formation_by_id)
            .configure(module_id::config),
    );
}
