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

/// MAIR-262: the dates an agent has not reached yet are `null` in the answers, so the contract
/// must say so. `value_type = String` on an `Option` dropped the `null`, and a client validating
/// the progress of an unfinished module against the spec refused it.
#[test]
fn unfinished_progress_dates_are_nullable_in_the_contract() {
    let document = serde_json::to_value(
        <elearning_api::endpoints::swagger::ApiDoc as utoipa::OpenApi>::openapi(),
    )
    .unwrap();
    let schemas = &document["components"]["schemas"];
    for (schema, field) in [
        ("UsersFormation", "completed_at"),
        ("UsersFormationModule", "completed_at"),
        ("UsersModuleContent", "finished_at"),
    ] {
        let property = &schemas[schema]["properties"][field];
        let nullable = property["type"]
            .as_array()
            .is_some_and(|types| types.iter().any(|t| t == "null"))
            || property["oneOf"]
                .as_array()
                .is_some_and(|variants| variants.iter().any(|v| v["type"] == "null"));
        assert!(nullable, "{schema}.{field} is not nullable: {property}");
    }
}
