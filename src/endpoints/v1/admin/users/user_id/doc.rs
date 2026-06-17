use crate::endpoints::v1::admin::users::user_id::formation_id::doc::FormationIdDoc;
use crate::endpoints::v1::admin::users::user_id::get::endpoint::__path_get_user_formations;
use crate::endpoints::v1::admin::users::user_id::get::view::GetUserByIdResultView;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{formation_id}", api = FormationIdDoc),
))]
pub struct UsersIdDoc;

#[derive(OpenApi)]
#[openapi(paths(get_user_formations), components(schemas(GetUserByIdResultView)))]
struct Doc;
