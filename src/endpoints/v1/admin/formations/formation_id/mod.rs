pub mod delete;
pub mod doc;
pub mod get;
pub mod learners;
pub mod patch;
pub mod register;
pub mod reminders;

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

/// Path parameters of the admin routes that replace or delete a formation. Same parameter as
/// [`AdminFormationIdParams`]; its example points to a formation of the test seed that nothing
/// else reads, so the security scan, which calls every operation on the examples, does not
/// delete the formation the other operations use.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminFormationWriteParams {
    /// Id of the formation.
    #[param(example = 7)]
    formation_id: u64,
}

impl AdminFormationWriteParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{formation_id}")
            .service(get::endpoint::get_formation_by_id)
            .service(register::endpoint::register_user_to_formation)
            .service(patch::endpoint::update_formation)
            .service(delete::endpoint::delete_formation)
            .service(reminders::endpoint::send_reminders)
            .service(learners::endpoint::get_formation_learners),
    );
}
