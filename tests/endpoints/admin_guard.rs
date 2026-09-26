//! MAIR-223: `/api/v1/admin/*` is reserved to admins (`AdminMiddleware`).

use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{create_course, create_user, enrol};
use crate::{init_app, status_of};

use elearning_api::database::formations::check_module_access::view::{
    CheckModuleAccessQueryView, ModuleAccessRow,
};

async fn is_enrolled(ctx: &TestContext, user_id: i32, course_id: i32) -> bool {
    let view = CheckModuleAccessQueryView::new(user_id as u64, course_id as u64, 0);
    ctx.db()
        .fetch_one::<ModuleAccessRow, _>(&view)
        .await
        .expect("query failed")
        .enrolled()
}

#[actix_web::test]
async fn admin_routes_reject_a_plain_agent_with_403() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let jwt = jwt_for(agent_id);

    let reads = [
        "/api/v1/admin/formations/".to_string(),
        format!("/api/v1/admin/formations/{course_id}/"),
        "/api/v1/admin/users/".to_string(),
        format!("/api/v1/admin/users/{agent_id}/"),
        format!("/api/v1/admin/users/{agent_id}/{course_id}/"),
    ];
    for uri in reads {
        let req = TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", jwt.as_str()));
        assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN, "GET {uri}");
    }
}

#[actix_web::test]
async fn plain_agent_cannot_enrol_itself() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;

    let req = TestRequest::post()
        .uri(&format!("/api/v1/admin/formations/{course_id}/"))
        .insert_header(("Authorization", jwt_for(agent_id)))
        .set_json(serde_json::json!({ "user_id": agent_id }));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    assert!(!is_enrolled(&ctx, agent_id, course_id).await);
}

#[actix_web::test]
async fn plain_agent_cannot_unenrol_someone_else() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;
    let victim_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    enrol(ctx.db(), victim_id, course_id).await;

    let req = TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{victim_id}/{course_id}/"))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    assert!(is_enrolled(&ctx, victim_id, course_id).await);
}

#[actix_web::test]
async fn admin_routes_require_a_jwt() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);

    let req = TestRequest::get().uri("/api/v1/admin/users/");
    assert_eq!(status_of!(app, req), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn admin_can_enrol_read_and_unenrol() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let admin_id = *ADMIN_ID.get().expect("Admin ID missing");
    let admin_jwt = jwt_for(admin_id);
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;

    let req = TestRequest::post()
        .uri(&format!("/api/v1/admin/formations/{course_id}/"))
        .insert_header(("Authorization", admin_jwt.as_str()))
        .set_json(serde_json::json!({ "user_id": agent_id }));
    assert!(status_of!(app, req).is_success());
    assert!(is_enrolled(&ctx, agent_id, course_id).await);

    let req = TestRequest::get()
        .uri("/api/v1/admin/users/")
        .insert_header(("Authorization", admin_jwt.as_str()));
    assert_eq!(status_of!(app, req), StatusCode::OK);

    let req = TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{agent_id}/{course_id}/"))
        .insert_header(("Authorization", admin_jwt.as_str()));
    assert!(status_of!(app, req).is_success());
    assert!(!is_enrolled(&ctx, agent_id, course_id).await);
}
