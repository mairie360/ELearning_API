use actix_web::{http::Method, test, web, App, HttpResponse};
use elearning_api::endpoints::config;
use elearning_api::endpoints::swagger::ApiDoc;
use mairie360_api_lib::state::AppState;
use utoipa::OpenApi;

// Every operation published in the OpenAPI contract (the one @mairie360/elearning-api-openapi is generated
// from) must hit an actix route that is really mounted. No database nor JWT is needed: a missing route
// falls through to the default service (418), a routed one fails further on (data, JWT, body).
// The lib's `AdminMiddleware` needs an `AppState` in the app data; it is built on an unreachable
// database (the connection failure is tolerated) since a request without JWT never reaches SQL.
#[actix_web::test]
async fn every_published_operation_is_routed() {
    let state = AppState::with_keycloak(
        "redis://127.0.0.1:1".to_string(),
        "postgres://user:password@127.0.0.1:1/db".to_string(),
        None,
    )
    .await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(web::scope("/api").configure(config))
            .default_service(web::to(HttpResponse::ImATeapot)),
    )
    .await;

    let document = serde_json::to_value(ApiDoc::openapi()).expect("serializable OpenAPI contract");
    let paths = document["paths"].as_object().expect("paths");
    let mut checked = 0;
    let mut unrouted = Vec::new();

    for (template, operations) in paths
        .iter()
        .filter(|(path, _)| path.starts_with("/api/v1/"))
    {
        if template.contains("//") {
            unrouted.push(format!("empty segment in {template}"));
        }
        let uri = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    "1"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/");

        for method in operations.as_object().expect("operations").keys() {
            let method = Method::from_bytes(method.to_uppercase().as_bytes()).expect("HTTP method");
            let request = test::TestRequest::default()
                .method(method.clone())
                .uri(&uri)
                .to_request();
            // A middleware rejection (401/403) comes back as an `Err`: the route exists.
            let status = match test::try_call_service(&app, request).await {
                Ok(response) => response.status(),
                Err(error) => error.as_response_error().status_code(),
            };
            checked += 1;
            if status.as_u16() == 418 {
                unrouted.push(format!("{method} {template}"));
            }
        }
    }

    // Guards the filter above: a wrong prefix would otherwise make the test pass on nothing.
    assert!(checked > 0, "no /api/v1 operation found in the contract");
    assert!(
        unrouted.is_empty(),
        "published operations without an actix route: {unrouted:?}"
    );
}
