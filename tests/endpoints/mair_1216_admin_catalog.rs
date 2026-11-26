//! MAIR-1216: `GET /api/v1/formations/catalog/` lists every formation to an administrator, enrolled
//! or not, so the front can open and manage them; an agent still only sees their enrolments.

use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::init_app;
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};

const URI: &str = "/api/v1/formations/catalog/";

fn find(body: &serde_json::Value, id: i32) -> Option<serde_json::Value> {
    body["formations"]
        .as_array()
        .expect("formations")
        .iter()
        .find(|f| f["id"] == id)
        .cloned()
}

#[actix_web::test]
async fn mair_1216_admin_catalog_lists_formations_the_admin_is_not_enrolled_in() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "Prise en main", "Découverte de l'outil").await;
    let module = create_module(ctx.db(), course, "Module 1", "Contenu 1", 1).await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, course).await;

    let req = TestRequest::get()
        .uri(URI)
        .insert_header((
            "Authorization",
            jwt_for(*ADMIN_ID.get().expect("Admin ID missing")),
        ))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let formation = find(&body, course).unwrap_or_else(|| panic!("course missing: {body}"));
    assert_eq!(formation["name"], "Prise en main");
    assert_eq!(formation["status"], "NotStarted");
    assert_eq!(formation["started_at"], serde_json::Value::Null);
    assert_eq!(
        formation["learners"], 1,
        "the admin is not counted as a learner"
    );
    assert_eq!(formation["modules"][0]["id"], module);
    assert_eq!(formation["modules"][0]["completed"], false);
}

#[actix_web::test]
async fn mair_1216_agent_catalog_still_lists_only_their_enrolments() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let mine = create_course(ctx.db(), "RGPD", "Comprendre le RGPD").await;
    let other = create_course(ctx.db(), "Accueil", "Accueil du public").await;
    let agent_id = create_user(ctx.db()).await;
    enrol(ctx.db(), agent_id, mine).await;

    let req = TestRequest::get()
        .uri(URI)
        .insert_header(("Authorization", jwt_for(agent_id)))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert!(find(&body, mine).is_some(), "{body}");
    assert!(
        find(&body, other).is_none(),
        "an agent never sees other formations: {body}"
    );
    assert_eq!(
        body["formations"].as_array().map(Vec::len),
        Some(1),
        "{body}"
    );
}
