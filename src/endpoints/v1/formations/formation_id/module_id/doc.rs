use crate::endpoints::v1::formations::formation_id::module_id::get::view::GetModuleResponseView;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct ModuleIdDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas(GetModuleResponseView)))]
struct Doc;
