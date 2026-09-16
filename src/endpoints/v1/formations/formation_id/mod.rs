pub mod doc;
pub mod get;
pub mod module_id;

/// Paramètres de chemin des routes d'une formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct FormationIdParams {
    /// Identifiant de la formation.
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
            .configure(module_id::config),
    );
}
