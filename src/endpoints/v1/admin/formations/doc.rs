use crate::endpoints::v1::admin::formations::formation_id::doc::IdDoc;
use crate::endpoints::v1::admin::formations::get::endpoint::__path_get_formations;
use crate::endpoints::v1::admin::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::admin::formations::post::endpoint::__path_create_formation;
use crate::endpoints::v1::admin::formations::post::view::{
    CourseBody, CourseIdView, FileBody, ModuleBody,
};
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
    paths(get_formations, create_formation),
    components(schemas(
        AdminFormation,
        AdminFormationModule,
        AdminModuleContent,
        GetFormationsResultView,
        CourseBody,
        ModuleBody,
        FileBody,
        CourseIdView
    ))
)]
struct Doc;
