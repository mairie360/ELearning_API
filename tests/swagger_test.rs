//! MAIR-424: Swagger UI and the served spec only exist with SWAGGER_ENABLED.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::App;
use elearning_api::endpoints::swagger::{self, SWAGGER_ENABLED_ENV};
use serial_test::serial;

async fn status_of(enabled: bool, uri: &str) -> StatusCode {
    let app = test::init_service(App::new().configure(|cfg| swagger::config(cfg, enabled))).await;
    test::call_service(&app, TestRequest::get().uri(uri).to_request())
        .await
        .status()
}

#[actix_web::test]
async fn spec_and_ui_are_only_mounted_when_enabled() {
    assert_eq!(
        status_of(true, "/api-docs/openapi.json").await,
        StatusCode::OK
    );
    assert!(status_of(true, "/swagger-ui/").await.is_success());

    assert_eq!(
        status_of(false, "/api-docs/openapi.json").await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status_of(false, "/swagger-ui/").await,
        StatusCode::NOT_FOUND
    );
}

#[test]
#[serial]
fn swagger_is_off_unless_explicitly_enabled() {
    std::env::remove_var(SWAGGER_ENABLED_ENV);
    assert!(!swagger::is_enabled());

    for (value, expected) in [
        ("true", true),
        ("1", true),
        ("TRUE", true),
        ("false", false),
        ("", false),
    ] {
        std::env::set_var(SWAGGER_ENABLED_ENV, value);
        assert_eq!(swagger::is_enabled(), expected, "SWAGGER_ENABLED={value:?}");
    }
    std::env::remove_var(SWAGGER_ENABLED_ENV);
}

#[test]
fn hello_route_is_gone_from_the_contract() {
    use utoipa::OpenApi;
    let spec = serde_json::to_value(swagger::ApiDoc::openapi()).unwrap();
    assert!(spec["paths"].get("/").is_none());
}
