//! MAIR-395: enrolment, unenrolment and the learner list, seen from the admin
//! routes.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{create_course, create_user, enrol};
use crate::{init_app, status_of};

fn admin_jwt() -> String {
    jwt_for(*ADMIN_ID.get().expect("Admin ID missing"))
}

#[actix_web::test]
async fn enrolling_an_unknown_user_or_in_an_unknown_formation_is_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let jwt = admin_jwt();
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;

    let req = TestRequest::post()
        .uri(&format!("/api/v1/admin/formations/{course_id}/"))
        .insert_header(("Authorization", jwt.as_str()))
        .set_json(serde_json::json!({ "user_id": 999_999 }));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);

    let req = TestRequest::post()
        .uri("/api/v1/admin/formations/999999/")
        .insert_header(("Authorization", jwt.as_str()))
        .set_json(serde_json::json!({ "user_id": agent_id }));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);

    // Enrolling twice is idempotent.
    for _ in 0..2 {
        let req = TestRequest::post()
            .uri(&format!("/api/v1/admin/formations/{course_id}/"))
            .insert_header(("Authorization", jwt.as_str()))
            .set_json(serde_json::json!({ "user_id": agent_id }));
        assert_eq!(status_of!(app, req), StatusCode::OK);
    }
}

#[actix_web::test]
async fn unenrolling_twice_answers_404_the_second_time() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let jwt = admin_jwt();
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    enrol(ctx.db(), agent_id, course_id).await;
    let uri = format!("/api/v1/admin/users/{agent_id}/{course_id}/");

    let req = TestRequest::delete()
        .uri(&uri)
        .insert_header(("Authorization", jwt.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::NO_CONTENT);

    let req = TestRequest::delete()
        .uri(&uri)
        .insert_header(("Authorization", jwt.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);

    let req = TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{agent_id}/999999/"))
        .insert_header(("Authorization", jwt.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn user_list_is_paginated() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let jwt = admin_jwt();
    for _ in 0..3 {
        create_user(ctx.db()).await;
    }

    let req = TestRequest::get()
        .uri("/api/v1/admin/users/?limit=2")
        .insert_header(("Authorization", jwt.as_str()))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["users"].as_array().expect("users array").len(), 2);

    let req = TestRequest::get()
        .uri("/api/v1/admin/users/?offset=1000000")
        .insert_header(("Authorization", jwt.as_str()))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(body["users"].as_array().expect("users array").is_empty());

    let req = TestRequest::get()
        .uri("/api/v1/admin/users/?limit=abc")
        .insert_header(("Authorization", jwt.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::BAD_REQUEST);
}
