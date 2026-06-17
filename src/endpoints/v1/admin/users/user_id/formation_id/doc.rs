use crate::endpoints::v1::admin::users::user_id::formation_id::delete::endpoint::__path_unsub_formation;
use crate::endpoints::v1::admin::users::user_id::formation_id::get::endpoint::__path_get_user_formation;
use crate::endpoints::v1::admin::users::user_id::formation_id::get::view::GetUserFormation;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_user_formation, unsub_formation),
    components(schemas(GetUserFormation))
)]
pub struct FormationIdDoc;
