//! MAIR-680: Core is the only signer of the session JWTs. This API holds Core's public key
//! (`JWT_PUBLIC_KEY`) and no `JWT_SECRET`: it accepts the `EdDSA` tokens signed with Core's key and
//! refuses every HS256 token, whatever secret signed it, so a service that leaks its configuration
//! can no longer forge an admin token for this API.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use jsonwebtoken::{encode, EncodingKey, Header};
use mairie360_api_lib::jwt_manager::Claims;
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;
use serial_test::serial;

use crate::endpoints::{jwt_for, now, TestContext};
use crate::queries::fixtures::{create_course, create_user};
use crate::{init_app, status_of};

const CATALOG: &str = "/api/v1/formations/catalog/";

/// HS256 token of `user_id` signed with `secret`: what any holder of the old shared secret signs.
fn hs256(user_id: i32, secret: &[u8]) -> String {
    let token = encode(
        &Header::default(),
        &Claims::new(&user_id.to_string(), "Admin", now() + 3600),
        &EncodingKey::from_secret(secret),
    )
    .unwrap();
    format!("Bearer {token}")
}

fn admin_id() -> i32 {
    *ADMIN_ID.get().expect("Admin ID missing")
}

#[actix_web::test]
#[serial]
async fn mair_680_runs_without_jwt_secret() {
    let _ctx = TestContext::new().await;
    assert!(std::env::var_os("JWT_SECRET").is_none());
    assert!(std::env::var_os("JWT_PRIVATE_KEY").is_none());
    assert!(std::env::var_os("JWT_PUBLIC_KEY").is_some());
}

#[actix_web::test]
#[serial]
async fn mair_680_eddsa_token_signed_with_core_key_is_accepted() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent = create_user(ctx.db()).await;

    let req = TestRequest::get()
        .uri(CATALOG)
        .insert_header(("Authorization", jwt_for(agent)));
    assert_eq!(status_of!(app, req), StatusCode::OK);
    let req = TestRequest::get()
        .uri("/api/v1/admin/users/")
        .insert_header(("Authorization", jwt_for(admin_id())));
    assert_eq!(status_of!(app, req), StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn mair_680_hs256_token_is_refused_without_jwt_secret() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);

    // The secrets the stacks and the test suites used to share, and any other one.
    for secret in [
        b"elearning-test-secret".as_slice(),
        b"b\"secret\"",
        b"any-secret-of-a-compromised-service-0123456789",
    ] {
        for uri in [CATALOG, "/api/v1/admin/users/"] {
            let req = TestRequest::get()
                .uri(uri)
                .insert_header(("Authorization", hs256(admin_id(), secret)));
            assert_eq!(
                status_of!(app, req),
                StatusCode::UNAUTHORIZED,
                "HS256 on {uri}"
            );
        }
    }
}

/// Business journey with Core's tokens: the administrator enrols an agent in a formation, the
/// agent finds it in their catalog, the administrator unenrols them; the administrator's HS256
/// token is refused on the same routes.
#[actix_web::test]
#[serial]
async fn mair_680_admin_enrols_and_unenrols_an_agent_with_core_tokens_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent = create_user(ctx.db()).await;
    let course = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let enrol_uri = format!("/api/v1/admin/formations/{course}/");
    let unenrol_uri = format!("/api/v1/admin/users/{agent}/{course}/");
    let legacy = hs256(admin_id(), b"b\"secret\"");

    let req = TestRequest::post()
        .uri(&enrol_uri)
        .insert_header(("Authorization", legacy.as_str()))
        .set_json(serde_json::json!({ "user_id": agent }));
    assert_eq!(status_of!(app, req), StatusCode::UNAUTHORIZED);

    let req = TestRequest::post()
        .uri(&enrol_uri)
        .insert_header(("Authorization", jwt_for(admin_id())))
        .set_json(serde_json::json!({ "user_id": agent }));
    assert_eq!(status_of!(app, req), StatusCode::OK);

    let req = TestRequest::get()
        .uri(CATALOG)
        .insert_header(("Authorization", jwt_for(agent)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["formations"][0]["id"], course, "{body}");

    let req = TestRequest::delete()
        .uri(&unenrol_uri)
        .insert_header(("Authorization", legacy.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::UNAUTHORIZED);

    let req = TestRequest::delete()
        .uri(&unenrol_uri)
        .insert_header(("Authorization", jwt_for(admin_id())));
    assert_eq!(status_of!(app, req), StatusCode::NO_CONTENT);

    let req = TestRequest::get()
        .uri(CATALOG)
        .insert_header(("Authorization", jwt_for(agent)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body, serde_json::json!({ "formations": [] }));
}
