pub mod certificate;
pub mod doc;
pub mod get;
pub mod module_id;
pub mod rating;

/// Path parameters of the routes of a formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct FormationIdParams {
    /// Id of the formation.
    #[param(example = 4)]
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
            // Before `module_id::config`, whose `/{module_id}` scope would otherwise take these
            // segments for an id.
            .service(rating::endpoint::rate_formation)
            .service(certificate::endpoint::get_certificate)
            .configure(module_id::config),
    );
}
