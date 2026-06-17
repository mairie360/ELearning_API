use crate::endpoints::v1::formations::formation_id::module_id::complete::endpoint::__path_complete_module;
use crate::endpoints::v1::formations::formation_id::module_id::get::endpoint::__path_get_module;
use crate::endpoints::v1::formations::formation_id::module_id::get::view::GetModuleResponseView;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct ModuleIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_module, complete_module),
    components(schemas(GetModuleResponseView))
)]
struct Doc;
