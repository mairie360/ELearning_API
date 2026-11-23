//! MAIR-817: the learners of a formation and the agent search of the administration, for the
//! enrolment screens of the front (through BFF_Elearning).

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;
use serde_json::Value;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{
    archive_user, complete, create_course, create_module, create_named_user, create_user, enrol,
};
use crate::{init_app, status_of};

fn admin() -> String {
    jwt_for(*ADMIN_ID.get().expect("seeded admin"))
}

fn marker() -> String {
    format!(
        "Mkr{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

macro_rules! admin_get {
    ($app:expr, $uri:expr) => {{
        let req = TestRequest::get()
            .uri($uri)
            .insert_header(("Authorization", admin()))
            .to_request();
        let body: Value = test::call_and_read_body_json(&$app, req).await;
        body
    }};
}

fn ids(list: &Value, key: &str) -> Vec<i64> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|item| item[key].as_i64().unwrap())
        .collect()
}

#[actix_web::test]
async fn the_learners_of_a_formation_are_paged_by_name_with_their_progress() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let db = ctx.db();
    let course = create_course(db, "Apprenants", "Apprenants").await;
    let first = create_module(db, course, "Module 1", "Contenu", 1).await;
    create_module(db, course, "Module 2", "Contenu", 2).await;
    let zola = create_named_user(db, "Emile", "Zola").await;
    let blum = create_named_user(db, "Anne", "Blum").await;
    let archived = create_named_user(db, "Aaron", "Aaron").await;
    for user in [zola, blum, archived] {
        enrol(db, user, course).await;
    }
    complete(db, zola, course, first).await;
    archive_user(db, archived).await;

    let body = admin_get!(app, &format!("/api/v1/admin/formations/{course}/learners/"));
    assert_eq!(body["formation_id"], course);
    assert_eq!(body["total"], 2, "{body}");
    assert_eq!(body["limit"], 50);
    assert_eq!(body["offset"], 0);
    assert_eq!(
        ids(&body["learners"], "user_id"),
        vec![i64::from(blum), i64::from(zola)]
    );
    let anne = &body["learners"][0];
    assert_eq!(anne["first_name"], "Anne");
    assert_eq!(anne["last_name"], "Blum");
    assert_eq!(anne["status"], "NotStarted");
    assert_eq!(anne["progress"], 0);
    for key in [
        "started_at",
        "completed_at",
        "last_activity_at",
        "last_reminder_at",
    ] {
        assert_eq!(anne[key], Value::Null, "{key}");
    }
    let emile = &body["learners"][1];
    assert_eq!(emile["status"], "InProgress");
    assert_eq!(emile["progress"], 50);
    assert!(emile["started_at"].as_str().unwrap().ends_with('Z'));
    assert!(emile["last_activity_at"].is_string());

    let page = admin_get!(
        app,
        &format!("/api/v1/admin/formations/{course}/learners/?limit=1&offset=1")
    );
    assert_eq!(page["total"], 2);
    assert_eq!(page["limit"], 1);
    assert_eq!(page["offset"], 1);
    assert_eq!(ids(&page["learners"], "user_id"), vec![i64::from(zola)]);

    let capped = admin_get!(
        app,
        &format!("/api/v1/admin/formations/{course}/learners/?limit=1000&offset=5")
    );
    assert_eq!(capped["limit"], 200);
    assert!(capped["learners"].as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn the_learners_route_refuses_what_it_documents() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "Refus", "Refus").await;
    let agent = jwt_for(create_user(ctx.db()).await);
    let uri = format!("/api/v1/admin/formations/{course}/learners/");

    let req = TestRequest::get()
        .uri(&uri)
        .insert_header(("Authorization", agent));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    assert_eq!(
        status_of!(app, TestRequest::get().uri(&uri)),
        StatusCode::UNAUTHORIZED
    );
    for (uri, status) in [
        (
            "/api/v1/admin/formations/2147483647/learners/".to_string(),
            StatusCode::NOT_FOUND,
        ),
        (
            "/api/v1/admin/formations/abc/learners/".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (format!("{uri}?limit=-1"), StatusCode::BAD_REQUEST),
        (format!("{uri}?offset=abc"), StatusCode::BAD_REQUEST),
    ] {
        let req = TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", admin()));
        assert_eq!(status_of!(app, req), status, "{uri}");
    }
}

#[actix_web::test]
async fn the_agents_list_searches_names_and_leaves_out_a_formation_learners() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let db = ctx.db();
    let tag = marker();
    let jean = create_named_user(db, "Jean", &format!("Dupont{tag}")).await;
    let amina = create_named_user(db, "Amina", &format!("Bensaid{tag}")).await;
    let course = create_course(db, "Exclusion", "Exclusion").await;
    enrol(db, amina, course).await;

    let body = admin_get!(
        app,
        &format!("/api/v1/admin/users/?search=%20{}%20", tag.to_lowercase())
    );
    assert_eq!(
        ids(&body["users"], "id"),
        vec![i64::from(jean), i64::from(amina)]
    );
    let first = &body["users"][0];
    assert_eq!(first["first_name"], "Jean");
    assert_eq!(first["last_name"], format!("Dupont{tag}"));
    assert_eq!(first["name"], format!("Jean Dupont{tag}"));

    let body = admin_get!(
        app,
        &format!("/api/v1/admin/users/?search={tag}&exclude_formation_id={course}")
    );
    assert_eq!(ids(&body["users"], "id"), vec![i64::from(jean)]);

    // Without a filter, every item carries the split names too.
    let body = admin_get!(app, "/api/v1/admin/users/?limit=5");
    for user in body["users"].as_array().unwrap() {
        assert!(user["first_name"].is_string() && user["last_name"].is_string());
    }
}

#[actix_web::test]
async fn the_agents_list_refuses_invalid_filters() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let long = "a".repeat(101);
    for query in [
        "search=".to_string(),
        "search=%20%20".to_string(),
        format!("search={long}"),
        "search=a%07b".to_string(),
        "exclude_formation_id=0".to_string(),
        "exclude_formation_id=2147483648".to_string(),
        "exclude_formation_id=abc".to_string(),
        "exclude_formation_id=-4".to_string(),
    ] {
        let req = TestRequest::get()
            .uri(&format!("/api/v1/admin/users/?{query}"))
            .insert_header(("Authorization", admin()));
        assert_eq!(status_of!(app, req), StatusCode::BAD_REQUEST, "{query}");
    }
    let req = TestRequest::get()
        .uri(&format!("/api/v1/admin/users/?search={}", "a".repeat(100)))
        .insert_header(("Authorization", admin()));
    assert_eq!(status_of!(app, req), StatusCode::OK);

    let agent = jwt_for(create_user(ctx.db()).await);
    let req = TestRequest::get()
        .uri("/api/v1/admin/users/?search=jean")
        .insert_header(("Authorization", agent));
    assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
}
