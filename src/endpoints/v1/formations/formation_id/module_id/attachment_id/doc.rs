use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::get::endpoint::__path_get_attachment_url;
use crate::endpoints::v1::formations::formation_id::module_id::attachment_id::get::view::GetAttachmentUrlView;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct AttachmentIdDoc;

#[derive(OpenApi)]
#[openapi(paths(get_attachment_url), components(schemas(GetAttachmentUrlView)))]
struct Doc;
