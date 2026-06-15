use crate::endpoints::v1::admin::formations::formation_id::get::endpoint::__path_get_formation_by_id;
use crate::endpoints::v1::admin::formations::formation_id::get::view::GetFormationByIdResultView;
use crate::endpoints::v1::admin::formations::formation_id::register::endpoint::__path_register_user_to_formation;
use crate::endpoints::v1::admin::formations::formation_id::register::view::RegisterUserView;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_formation_by_id, register_user_to_formation),
    components(schemas(GetFormationByIdResultView, RegisterUserView))
)]
pub struct IdDoc;
