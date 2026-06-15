use crate::endpoints::v1::admin::formations::doc::FormationsDoc;
use crate::endpoints::v1::admin::users::doc::UsersDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/users", api = UsersDoc),
    (path = "/formations", api = FormationsDoc)
))]
pub struct AdminDoc;
