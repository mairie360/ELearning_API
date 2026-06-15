use crate::endpoints::v1::admin::users::id::id::doc::FormationIdDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{formation_id}", api = FormationIdDoc),
))]
pub struct UsersIdDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas()))]
struct Doc;
