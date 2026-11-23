use crate::endpoints::v1::formations::formation_id::certificate::endpoint::__path_get_certificate;
use crate::endpoints::v1::formations::formation_id::get::endpoint::__path_get_my_formation_by_id;
use crate::endpoints::v1::formations::formation_id::get::view::GetFormationResponseView;
use crate::endpoints::v1::formations::formation_id::module_id::doc::ModuleIdDoc;
use crate::endpoints::v1::formations::formation_id::rating::endpoint::__path_rate_formation;
use crate::endpoints::v1::formations::formation_id::rating::view::{
    RateFormationView, RatingResultView,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/rating/", api = RatingDoc),
    (path = "/certificate/", api = CertificateDoc),
    (path = "/{module_id}", api = ModuleIdDoc)
))]
pub struct FormationIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_my_formation_by_id),
    components(schemas(GetFormationResponseView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(
    paths(rate_formation),
    components(schemas(RateFormationView, RatingResultView))
)]
struct RatingDoc;

#[derive(OpenApi)]
#[openapi(paths(get_certificate))]
struct CertificateDoc;
