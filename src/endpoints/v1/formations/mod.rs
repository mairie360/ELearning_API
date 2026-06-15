pub mod doc;
pub mod formation_id;
pub mod get;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/formations")
            .service(get::endpoint::get_my_formations)
            .configure(formation_id::config),
    );
}
