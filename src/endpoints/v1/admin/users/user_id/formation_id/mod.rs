pub mod delete;
pub mod doc;
pub mod get;

/// Path parameters of the admin routes of a user's enrolment in a formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminUserFormationIdParams {
    /// Core API id of the user.
    #[param(example = 42)]
    user_id: u64,
    /// Id of the formation.
    #[param(example = 4)]
    formation_id: u64,
}

impl AdminUserFormationIdParams {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{formation_id}")
            .service(get::endpoint::get_user_formation)
            .service(delete::endpoint::unsub_formation),
    );
}
