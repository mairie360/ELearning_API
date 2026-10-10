//! MAIR-506: `GET /api/v1/formations/catalog/` returns the caller's formations with their modules
//! and files in one answer, and only the caller's.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{
    create_attachment, create_course, create_module, create_user, enrol,
};
use crate::{init_app, status_of};

const URI: &str = "/api/v1/formations/catalog/";

#[actix_web::test]
async fn catalog_requires_a_token() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);

    let req = TestRequest::get().uri(URI);
    assert_eq!(status_of!(app, req), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn catalog_is_empty_without_enrolment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;

    let req = TestRequest::get()
        .uri(URI)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body, serde_json::json!({ "formations": [] }));
}

#[actix_web::test]
async fn catalog_nests_modules_and_files_of_the_callers_formations_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let mine = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let other = create_course(ctx.db(), "Accueil", "Accueil du public").await;
    let second = create_module(ctx.db(), mine, "Module 2", "Contenu 2", 2).await;
    let first = create_module(ctx.db(), mine, "Module 1", "Contenu 1", 1).await;
    let foreign = create_module(ctx.db(), other, "Module X", "Contenu X", 1).await;
    let pdf = create_attachment(ctx.db(), first, "principes.pdf", "pdf", "s3/key-1", 4096).await;
    let video = create_attachment(ctx.db(), first, "intro.mp4", "video", "s3/key-2", 8192).await;
    create_attachment(ctx.db(), foreign, "secret.pdf", "pdf", "s3/key-3", 1).await;
    let agent_id = create_user(ctx.db()).await;
    let neighbour_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, mine).await;
    enrol(ctx.db(), neighbour_id, other).await;

    let req = TestRequest::get()
        .uri(URI)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let formations = body["formations"].as_array().expect("formations");
    assert_eq!(formations.len(), 1, "only the caller's enrolments: {body}");
    assert_eq!(formations[0]["id"], mine);
    assert_eq!(formations[0]["name"], "RGPD");
    assert_eq!(formations[0]["status"], "NotStarted");
    let modules = formations[0]["modules"].as_array().expect("modules");
    // Display order (`sort_order`), not creation order.
    assert_eq!(modules[0]["id"], first);
    assert_eq!(modules[1]["id"], second);
    assert_eq!(modules[0]["completed"], false);
    assert_eq!(modules[1]["files"], serde_json::json!([]));
    let files = modules[0]["files"].as_array().expect("files");
    assert_eq!(files[0]["id"], pdf);
    assert_eq!(files[0]["file_type"], "Pdf");
    assert_eq!(files[0]["file_size_bytes"], 4096);
    assert_eq!(files[1]["id"], video);
    assert_eq!(files[1]["file_type"], "Video");
    // The S3 key is never exposed, and nothing of the other learner's formation leaks.
    let text = body.to_string();
    assert!(!text.contains("s3/key"), "{text}");
    assert!(!text.contains("secret.pdf"), "{text}");
}

#[actix_web::test]
async fn catalog_matches_the_per_formation_routes() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let module = create_module(ctx.db(), course, "Module 1", "Contenu 1", 1).await;
    create_attachment(ctx.db(), module, "principes.pdf", "pdf", "s3/key-1", 4096).await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, course).await;
    let auth = jwt_for(agent_id);

    let req = TestRequest::get()
        .uri(URI)
        .insert_header(("Authorization", auth.clone()))
        .to_request();
    let catalog: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let req = TestRequest::get()
        .uri(&format!("/api/v1/formations/{course}/"))
        .insert_header(("Authorization", auth.clone()))
        .to_request();
    let modules: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let req = TestRequest::get()
        .uri(&format!("/api/v1/formations/{course}/{module}/"))
        .insert_header(("Authorization", auth))
        .to_request();
    let files: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let nested = &catalog["formations"][0]["modules"][0];
    assert_eq!(nested["id"], modules["modules"][0]["id"]);
    assert_eq!(nested["name"], modules["modules"][0]["name"]);
    assert_eq!(nested["completed"], modules["modules"][0]["completed"]);
    assert_eq!(nested["files"], files["files"]);
}
