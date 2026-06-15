use crate::endpoints::v1::admin::doc::AdminDoc;
use crate::endpoints::v1::formations::doc::FormationsDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/admin", api = AdminDoc),
    (path = "/formations", api = FormationsDoc)
))]
pub struct V1Doc;
