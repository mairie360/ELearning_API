//! MAIR-419: negative access tests, now that `endpoints/` counts towards the
//! coverage gate. Each test proves that a request a caller is not entitled to
//! is refused *and* leaks nothing.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::BOB_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use crate::{init_app, status_of};

#[actix_web::test]
async fn percent_encoded_admin_paths_are_still_admin_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;
    let jwt = jwt_for(agent_id);

    // `%61` is `a`, `%2E%2E` is `..`: none of these spellings may reach an admin
    // handler without going through `AdminMiddleware`.
    for uri in [
        "/api/v1/%61dmin/users/",
        "/api/v1/%61%64%6D%69%6E/formations/",
        "/api/v1/ADMIN/users/",
        "/api/v1//admin/users/",
        "/api/v1/formations/../admin/users/",
        "/api/v1/formations/%2E%2E/admin/users/",
    ] {
        let req = TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", jwt.as_str()));
        let status = status_of!(app, req);
        assert!(status.is_client_error(), "GET {uri} answered {status}");
    }
}

#[actix_web::test]
async fn archived_account_is_refused_even_with_a_valid_jwt() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    // Bob is seeded archived by the lib's test setup.
    let bob_id = *BOB_ID.get().expect("Bob ID missing");
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    enrol(ctx.db(), bob_id, course_id).await;

    for uri in [
        "/api/v1/formations/".to_string(),
        format!("/api/v1/formations/{course_id}/"),
    ] {
        let req = TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", jwt_for(bob_id)));
        // The lib's `JwtMiddleware` answers `404 Utilisateur inconnu` for an
        // account that no longer exists or is archived.
        assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND, "GET {uri}");
    }
}

#[actix_web::test]
async fn an_agent_never_sees_the_formations_of_another_agent() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner_id = create_user(ctx.db()).await;
    let other_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    enrol(ctx.db(), owner_id, course_id).await;

    let req = TestRequest::get()
        .uri("/api/v1/formations/")
        .insert_header(("Authorization", jwt_for(owner_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["formations"].as_array().unwrap().len(), 1);
    assert_eq!(body["formations"][0]["id"], course_id);
    assert_eq!(body["formations"][0]["status"], "NotStarted");

    let req = TestRequest::get()
        .uri("/api/v1/formations/")
        .insert_header(("Authorization", jwt_for(other_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["formations"], serde_json::json!([]));
}

#[actix_web::test]
async fn an_agent_cannot_complete_a_module_for_another_agent() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner_id = create_user(ctx.db()).await;
    let other_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(ctx.db(), course_id, "Module 1", "Contenu 1", 1).await;
    enrol(ctx.db(), owner_id, course_id).await;

    // The user always comes from the JWT: there is no id in the path or body
    // to point at someone else, so the other agent is simply not enrolled.
    let req = TestRequest::patch()
        .uri(&format!("/api/v1/formations/{course_id}/{module_id}/"))
        .insert_header(("Authorization", jwt_for(other_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);

    let req = TestRequest::get()
        .uri(&format!("/api/v1/formations/{course_id}/"))
        .insert_header(("Authorization", jwt_for(owner_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["modules"][0]["completed"], false);
}

/// The spec promises `400` for a path segment that is not an integer, while
/// actix's default `Path` extractor answers `404`.
#[actix_web::test]
async fn malformed_path_ids_are_400() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;

    for uri in [
        "/api/v1/formations/abc/",
        "/api/v1/formations/-1/",
        "/api/v1/formations/1/abc/",
    ] {
        let req = TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", jwt_for(agent_id)));
        assert_eq!(status_of!(app, req), StatusCode::BAD_REQUEST, "GET {uri}");
    }
}
