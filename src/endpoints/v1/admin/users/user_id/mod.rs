pub mod doc;
pub mod formation_id;
pub mod get;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminUserIdParams {
    user_id: u64,
}

impl AdminUserIdParams {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{user_id}")
            .service(get::endpoint::get_user_formations)
            .configure(formation_id::config),
    );
}
