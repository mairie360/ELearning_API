//! MAIR-223: the routes of a module require the caller to be enrolled in the
//! formation (403) and the module to belong to it (404).

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext, PRESIGNED_URL};
use crate::queries::fixtures::{
    create_attachment, create_course, create_module, create_user, enrol,
};
use crate::{init_app, status_of};

use elearning_api::database::formations::get_my_formation_modules::view::{
    FormationModuleRow, GetMyFormationModulesQueryView,
};

/// A course with one module and one attachment, plus a module of another
/// course to probe the formation/module coherence check.
struct Course {
    id: i32,
    module_id: i32,
    attachment_id: i32,
    foreign_module_id: i32,
    foreign_attachment_id: i32,
}

async fn seed_course(ctx: &TestContext) -> Course {
    let db = ctx.db();
    let id = create_course(db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(db, id, "Module 1", "Contenu 1", 1).await;
    let attachment_id =
        create_attachment(db, module_id, "guide.pdf", "pdf", "courses/guide.pdf", 1024).await;
    let other_id = create_course(db, "Autre", "Autre formation").await;
    let foreign_module_id = create_module(db, other_id, "Module X", "Contenu X", 1).await;
    let foreign_attachment_id = create_attachment(
        db,
        foreign_module_id,
        "secret.pdf",
        "pdf",
        "courses/secret.pdf",
        2048,
    )
    .await;
    Course {
        id,
        module_id,
        attachment_id,
        foreign_module_id,
        foreign_attachment_id,
    }
}

async fn is_completed(ctx: &TestContext, user_id: i32, course_id: i32, module_id: i32) -> bool {
    let view = GetMyFormationModulesQueryView::new(course_id as u64, user_id as u64);
    ctx.db()
        .fetch_all::<FormationModuleRow, _>(&view)
        .await
        .expect("query failed")
        .iter()
        .any(|row| row.id() == module_id && row.completed())
}

// ---------------------------------------------------------------------------
// GET /api/v1/formations/{formation_id}/{module_id}/
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn get_module_requires_enrolment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    let uri = format!("/api/v1/formations/{}/{}/", course.id, course.module_id);

    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);

    enrol(ctx.db(), agent_id, course.id).await;
    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["files"][0]["id"], course.attachment_id);
}

#[actix_web::test]
async fn get_module_of_another_formation_is_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, course.id).await;

    let req = TestRequest::get()
        .uri(&format!(
            "/api/v1/formations/{}/{}/",
            course.id, course.foreign_module_id
        ))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn get_module_gives_admins_no_bypass() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let admin_id = *ADMIN_ID.get().expect("Admin ID missing");

    let req = TestRequest::get()
        .uri(&format!(
            "/api/v1/formations/{}/{}/",
            course.id, course.module_id
        ))
        .insert_header(("Authorization", jwt_for(admin_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn get_attachment_url_requires_enrolment_and_signs_nothing_otherwise() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    let uri = format!(
        "/api/v1/formations/{}/{}/{}/",
        course.id, course.module_id, course.attachment_id
    );

    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    assert!(ctx.storage.calls.lock().unwrap().is_empty());

    enrol(ctx.db(), agent_id, course.id).await;
    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["url"], PRESIGNED_URL);
    assert_eq!(
        *ctx.storage.calls.lock().unwrap(),
        vec!["courses/guide.pdf".to_string()]
    );
}

#[actix_web::test]
async fn get_attachment_url_outside_the_formation_is_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, course.id).await;

    // Enrolled in `course`, asking for the attachment of another course
    // through a module of that other course.
    let req = TestRequest::get()
        .uri(&format!(
            "/api/v1/formations/{}/{}/{}/",
            course.id, course.foreign_module_id, course.foreign_attachment_id
        ))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);

    // ... or through a module of `course`.
    let req = TestRequest::get()
        .uri(&format!(
            "/api/v1/formations/{}/{}/{}/",
            course.id, course.module_id, course.foreign_attachment_id
        ))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);
    assert!(ctx.storage.calls.lock().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// PATCH /api/v1/formations/{formation_id}/{module_id}/ (complete)
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn complete_module_requires_enrolment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    let uri = format!("/api/v1/formations/{}/{}/", course.id, course.module_id);

    let req = TestRequest::patch()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    assert!(!is_completed(&ctx, agent_id, course.id, course.module_id).await);

    enrol(ctx.db(), agent_id, course.id).await;
    let req = TestRequest::patch()
        .uri(&uri)
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::OK);
    assert!(is_completed(&ctx, agent_id, course.id, course.module_id).await);
}

#[actix_web::test]
async fn complete_module_of_another_formation_is_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = seed_course(&ctx).await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, course.id).await;

    let req = TestRequest::patch()
        .uri(&format!(
            "/api/v1/formations/{}/{}/",
            course.id, course.foreign_module_id
        ))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);

    let req = TestRequest::patch()
        .uri(&format!("/api/v1/formations/{}/999999/", course.id))
        .insert_header(("Authorization", jwt_for(agent_id)));
    assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND);
}
