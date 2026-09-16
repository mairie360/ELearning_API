use crate::endpoints::v1::admin::users::UsersFormation;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetUserByIdResultView {
    /// Formations auxquelles l'agent est inscrit. Vide s'il n'en suit aucune, ou si l'identifiant
    /// ne correspond à personne.
    pub formations: Vec<UsersFormation>,
}
