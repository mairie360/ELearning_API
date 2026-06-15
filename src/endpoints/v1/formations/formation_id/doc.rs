use crate::endpoints::v1::formations::formation_id::get::view::GetFormationResponseView;
use crate::endpoints::v1::formations::formation_id::module_id::doc::ModuleIdDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{module_id}", api = ModuleIdDoc)
))]
pub struct FormationIdDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas(GetFormationResponseView)))]
struct Doc;
