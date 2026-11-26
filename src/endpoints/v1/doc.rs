use crate::endpoints::v1::admin::doc::AdminDoc;
use crate::endpoints::v1::formations::doc::FormationsDoc;
use crate::endpoints::v1::search::doc::SearchDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/admin", api = AdminDoc),
    (path = "/formations", api = FormationsDoc),
    (path = "/search", api = SearchDoc)
))]
pub struct V1Doc;
