pub mod health;
pub mod ready;
pub mod swagger;
pub mod v1;

use actix_web::{error, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    // actix answers `404` when a path segment does not deserialize (`abc` for a
    // `u64` id); the contract documents `400` for it.
    cfg.app_data(
        web::PathConfig::default().error_handler(|err, _| error::ErrorBadRequest(err.to_string())),
    );
    cfg.configure(v1::config);
}
