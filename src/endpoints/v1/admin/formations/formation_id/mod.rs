pub mod doc;
pub mod get;
pub mod register;

/// Path parameters of the admin routes of a formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminFormationIdParams {
    /// Id of the formation.
    #[param(example = 4)]
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
