pub mod admin;
pub mod doc;
pub mod formations;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1"));
}
