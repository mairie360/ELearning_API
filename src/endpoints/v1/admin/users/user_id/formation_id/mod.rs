pub mod delete;
pub mod doc;
pub mod get;

/// Paramètres de chemin des routes d'administration de l'inscription d'un utilisateur à une formation.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AdminUserFormationIdParams {
    /// Identifiant Core API de l'utilisateur.
    #[param(example = 42)]
    user_id: u64,
    /// Identifiant de la formation.
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
