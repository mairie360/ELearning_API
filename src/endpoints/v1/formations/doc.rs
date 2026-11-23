use crate::endpoints::v1::formations::catalog::endpoint::__path_get_my_catalog;
use crate::endpoints::v1::formations::catalog::view::{
    CatalogFile, CatalogFormation, CatalogModule, ContentKind, CourseLevel, CourseSource,
    GetCatalogResultView,
};
use crate::endpoints::v1::formations::certificates::endpoint::__path_get_my_certificates;
use crate::endpoints::v1::formations::formation_id::doc::FormationIdDoc;
use crate::endpoints::v1::formations::get::endpoint::__path_get_my_formations;
use crate::endpoints::v1::formations::get::view::GetFormationsResultView;
use crate::endpoints::v1::formations::summary::endpoint::__path_get_my_summary;
use crate::endpoints::v1::formations::summary::view::{SummaryFormation, SummaryResultView};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/catalog/", api = CatalogDoc),
    (path = "/summary/", api = SummaryDoc),
    (path = "/certificates/", api = CertificatesDoc),
    (path = "/{formation_id}", api = FormationIdDoc)
))]
pub struct FormationsDoc;

#[derive(OpenApi)]
#[openapi(paths(get_my_formations), components(schemas(GetFormationsResultView)))]
struct Doc;

#[derive(OpenApi)]
#[openapi(
    paths(get_my_catalog),
    components(schemas(
        GetCatalogResultView,
        CatalogFormation,
        CatalogModule,
        CatalogFile,
        ContentKind,
        CourseLevel,
        CourseSource
    ))
)]
struct CatalogDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_my_summary),
    components(schemas(SummaryResultView, SummaryFormation))
)]
struct SummaryDoc;

#[derive(OpenApi)]
#[openapi(paths(get_my_certificates))]
struct CertificatesDoc;
