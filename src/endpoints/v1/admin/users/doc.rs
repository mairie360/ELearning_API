use crate::endpoints::v1::admin::users::id::doc::UsersIdDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{user_id}", api = UsersIdDoc),
))]
pub struct UsersDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas()))]
struct Doc;
