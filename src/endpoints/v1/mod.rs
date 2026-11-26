pub mod admin;
pub mod doc;
pub mod formations;
pub mod search;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            .configure(formations::config)
            .configure(admin::config)
            .service(web::scope("/search").configure(search::config)),
    );
}
