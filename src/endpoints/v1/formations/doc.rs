use crate::endpoints::v1::formations::formation_id::doc::FormationIdDoc;
use crate::endpoints::v1::formations::get::view::GetFormationsResultView;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{formation_id}", api = FormationIdDoc)
))]
pub struct FormationsDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas(GetFormationsResultView)))]
struct Doc;
