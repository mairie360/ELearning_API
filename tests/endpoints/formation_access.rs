//! MAIR-223: `GET /api/v1/formations/{formation_id}/` requires the caller to be
//! enrolled in the formation; an unknown formation answers `403` as well.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use crate::{init_app, status_of};

#[actix_web::test]
async fn get_formation_requires_enrolment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(ctx.db(), course_id, "Module 1", "Contenu 1", 1).await;
    let agent_id = create_user(ctx.db()).await;
    let uri = format!("/api/v1/formations/{course_id}/");

    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);

    enrol(ctx.db(), agent_id, course_id).await;
    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["modules"][0]["id"], module_id);
    assert_eq!(body["modules"][0]["completed"], false);
}

#[actix_web::test]
async fn get_unknown_formation_is_403_not_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;

    let req = TestRequest::get()
        .uri("/api/v1/formations/999999/")
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn get_formation_gives_admins_no_bypass() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let admin_id = *ADMIN_ID.get().expect("Admin ID missing");

    let req = TestRequest::get()
        .uri(&format!("/api/v1/formations/{course_id}/"))
        .insert_header(("Authorization", jwt_for(admin_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
}
