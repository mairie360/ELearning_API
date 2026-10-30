pub mod catalog;
pub mod doc;
pub mod formation_id;
pub mod get;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/formations")
            .service(get::endpoint::get_my_formations)
            // Before `formation_id::config`, whose `/{formation_id}` scope would otherwise take "catalog".
            .service(catalog::endpoint::get_my_catalog)
            .configure(formation_id::config),
    );
}
