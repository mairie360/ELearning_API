use crate::endpoints::v1::admin::formations::id::get::view::GetFormationByIdResultView;
use crate::endpoints::v1::admin::formations::id::register::view::RegisterUserView;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas(GetFormationByIdResultView, RegisterUserView))
)]
pub struct IdDoc;
