use crate::endpoints::v1::admin::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::admin::formations::id::doc::IdDoc;
use crate::endpoints::v1::admin::formations::{
    AdminFormation, AdminFormationModule, AdminModuleContent,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{formation_id}", api = IdDoc),
))]
pub struct FormationsDoc;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas(
        AdminFormation,
        AdminFormationModule,
        AdminModuleContent,
        GetFormationsResultView
    ))
)]
struct Doc;
