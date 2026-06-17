use crate::endpoints::v1::admin::users::get::endpoint::__path_get_users;
use crate::endpoints::v1::admin::users::get::view::GetUsersResultView;
use crate::endpoints::v1::admin::users::user_id::doc::UsersIdDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{user_id}", api = UsersIdDoc),
))]
pub struct UsersDoc;

#[derive(OpenApi)]
#[openapi(paths(get_users), components(schemas(GetUsersResultView)))]
struct Doc;
