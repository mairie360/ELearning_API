pub mod doc;
pub mod get;
pub mod register;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminFormationIdParams {
    formation_id: u64,
}

impl AdminFormationIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{formation_id}")
            .service(get::endpoint::get_formation_by_id)
            .service(register::endpoint::register_user_to_formation),
    );
}
