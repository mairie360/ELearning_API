//! MAIR-1228: `GET /api/v1/search`, ELearning's part of the global search of the shared header
//! (formations of the caller's catalogue). Each test names its formations with a unique marker
//! used as the term.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;
use serde_json::Value;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{create_course, create_user, enrol};
use crate::{init_app, status_of};

/// A marker made of letters only, unique to the test.
fn marker(prefix: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = format!(
        "{nanos}{}",
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let letters: String = unique
        .bytes()
        .map(|digit| char::from(b'a' + (digit - b'0')))
        .collect();
    format!("{prefix}{letters}")
}

fn search(query: &str, user_id: i32) -> TestRequest {
    TestRequest::get()
        .uri(&format!("/api/v1/search{query}"))
        .insert_header(("Authorization", jwt_for(user_id)))
}

fn ids(body: &Value) -> Vec<i64> {
    body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["id"].as_i64().unwrap())
        .collect()
}

#[actix_web::test]
async fn mair_1228_finds_the_formations_of_the_catalogue_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let term = marker("accueil");
    let starting = create_course(ctx.db(), &format!("{term} du public"), "Bonnes pratiques").await;
    let inner = create_course(ctx.db(), &format!("Module {term}"), "").await;
    let learner = create_user(ctx.db()).await;
    let outsider = create_user(ctx.db()).await;
    enrol(ctx.db(), learner, starting).await;
    enrol(ctx.db(), learner, inner).await;
    let admin = *ADMIN_ID.get().expect("Admin ID missing");

    let body: Value =
        test::call_and_read_body_json(&app, search(&format!("?q={term}"), learner).to_request())
            .await;
    // The title starting with the term comes first.
    assert_eq!(
        ids(&body),
        vec![i64::from(starting), i64::from(inner)],
        "{body}"
    );
    let first = &body["items"][0];
    assert_eq!(first["kind"], "course");
    assert_eq!(first["title"], format!("{term} du public"));
    assert_eq!(first["excerpt"], "Bonnes pratiques");
    assert!(first["parent_id"].is_null());
    assert!(first["updated_at"].as_str().unwrap().ends_with('Z'));
    assert!(
        body["items"][1]["excerpt"].is_null(),
        "no description: {body}"
    );

    let body: Value =
        test::call_and_read_body_json(&app, search(&format!("?q={term}"), outsider).to_request())
            .await;
    assert!(ids(&body).is_empty(), "not enrolled: {body}");
    let body: Value =
        test::call_and_read_body_json(&app, search(&format!("?q={term}"), admin).to_request())
            .await;
    assert_eq!(
        ids(&body).len(),
        2,
        "an administrator sees every formation: {body}"
    );
}

#[actix_web::test]
async fn mair_1228_limit_cuts_and_wildcards_match_themselves() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let term = marker("rgpd");
    let learner = create_user(ctx.db()).await;
    for index in 0..3 {
        let course = create_course(ctx.db(), &format!("Lot {index} {term}"), "").await;
        enrol(ctx.db(), learner, course).await;
    }

    let body: Value = test::call_and_read_body_json(
        &app,
        search(&format!("?q={term}&limit=2"), learner).to_request(),
    )
    .await;
    assert_eq!(ids(&body).len(), 2, "{body}");
    // Unescaped, `%_<term>` would match the formations above (any character before the term).
    let body: Value = test::call_and_read_body_json(
        &app,
        search(&format!("?q=%25_{term}"), learner).to_request(),
    )
    .await;
    assert!(ids(&body).is_empty(), "{body}");
}

#[actix_web::test]
async fn mair_1228_rejects_a_short_term_a_bad_limit_and_no_session() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let learner = create_user(ctx.db()).await;
    for query in [
        "?q=a",
        "?q=%20a%20",
        "?q=%0Aab",
        "",
        "?q=ab&limit=21",
        "?q=ab&limit=0",
        "?q=ab&limit=abc",
        "?q=ab&limit=-1",
    ] {
        let request = TestRequest::get()
            .uri(&format!("/api/v1/search{query}"))
            .insert_header(("Authorization", jwt_for(learner)));
        assert_eq!(status_of!(app, request), StatusCode::BAD_REQUEST, "{query}");
    }
    let anonymous = TestRequest::get().uri("/api/v1/search?q=accueil");
    assert_eq!(status_of!(app, anonymous), StatusCode::UNAUTHORIZED);
}
