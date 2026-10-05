//! MAIR-419: the admin read routes, success and not-found branches.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{
    create_attachment, create_course, create_module, create_user, enrol,
};
use crate::{init_app, status_of};

fn admin_jwt() -> String {
    jwt_for(*ADMIN_ID.get().expect("Admin ID missing"))
}

/// GETs `$uri` as the admin and decodes the JSON body.
macro_rules! get_json {
    ($app:expr, $uri:expr) => {{
        let req = TestRequest::get()
            .uri($uri)
            .insert_header(("Authorization", admin_jwt()))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&$app, req).await;
        body
    }};
}

#[actix_web::test]
async fn admin_reads_the_catalogue_and_a_formation() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(ctx.db(), course_id, "Module 1", "Contenu 1", 1).await;
    let attachment_id = create_attachment(
        ctx.db(),
        module_id,
        "cours.pdf",
        "pdf",
        "rgpd/cours.pdf",
        1024,
    )
    .await;

    let body = get_json!(app, "/api/v1/admin/formations/?details=true");
    let course = body["formations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == course_id)
        .expect("created course missing from the catalogue");
    assert_eq!(course["modules"][0]["id"], module_id);

    let body = get_json!(app, &format!("/api/v1/admin/formations/{course_id}/"));
    assert_eq!(body["modules"][0]["id"], module_id);
    assert_eq!(body["modules"][0]["content"], serde_json::Value::Null);

    let body = get_json!(
        app,
        &format!("/api/v1/admin/formations/{course_id}/?details=true")
    );
    assert_eq!(body["modules"][0]["content"][0]["id"], attachment_id);
}

#[actix_web::test]
async fn admin_reads_the_progress_of_an_agent() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;
    let course_id = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(ctx.db(), course_id, "Module 1", "Contenu 1", 1).await;
    enrol(ctx.db(), agent_id, course_id).await;

    let body = get_json!(
        app,
        &format!("/api/v1/admin/users/{agent_id}/?details=true")
    );
    assert_eq!(body["formations"][0]["id"], course_id);
    assert_eq!(body["formations"][0]["progress_status"], "NotStarted");
    assert_eq!(body["formations"][0]["modules"][0]["id"], module_id);

    let body = get_json!(app, &format!("/api/v1/admin/users/{agent_id}/{course_id}/"));
    assert_eq!(body["modules"][0]["id"], module_id);
    assert_eq!(body["modules"][0]["is_completed"], false);
}

#[actix_web::test]
async fn admin_reads_of_unknown_rows() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent_id = create_user(ctx.db()).await;

    for uri in [
        "/api/v1/admin/formations/999999/".to_string(),
        format!("/api/v1/admin/users/{agent_id}/999999/"),
    ] {
        let req = TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", admin_jwt()));
        assert_eq!(status_of!(app, req), StatusCode::NOT_FOUND, "GET {uri}");
    }

    // An agent without enrolment has an empty progress, not an error.
    let body = get_json!(app, &format!("/api/v1/admin/users/{agent_id}/"));
    assert_eq!(body["formations"], serde_json::json!([]));
}
