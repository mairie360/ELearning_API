use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas()))]
pub struct FormationIdDoc;
