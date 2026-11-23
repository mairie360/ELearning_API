//! MAIR-1150, MAIR-1149, MAIR-1144, MAIR-1143: the learner side of the E-learning V2: the full
//! catalogue, the ratings, the training time of the year and the attestations.

use actix_web::http::{header, StatusCode};
use actix_web::test::{self, TestRequest};
use chrono::Datelike;
use serde_json::{json, Value};

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{
    complete, create_attachment, create_course, create_module, create_user, enrol,
    set_attachment_duration, set_course_meta,
};
use crate::{init_app, status_of};

/// A course of one module holding a 10-minute PDF and a 5-minute link.
struct Course {
    id: i32,
    module: i32,
}

async fn course(ctx: &TestContext, title: &str) -> Course {
    let id = create_course(ctx.db(), title, "Description").await;
    let module = create_module(ctx.db(), id, "Module 1", "Contenu", 1).await;
    let pdf = create_attachment(ctx.db(), module, "guide.pdf", "pdf", "s3/secret-key", 2048).await;
    let link = create_attachment(
        ctx.db(),
        module,
        "Texte officiel",
        "link",
        "https://example.org/texte",
        0,
    )
    .await;
    set_attachment_duration(ctx.db(), pdf, 600).await;
    set_attachment_duration(ctx.db(), link, 300).await;
    Course { id, module }
}

fn this_year() -> i32 {
    chrono::Utc::now().year()
}

#[actix_web::test]
async fn the_catalogue_carries_the_metadata_of_the_v2() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = course(&ctx, "Catalogue V2").await;
    set_course_meta(ctx.db(), course.id, true, "2030-06-30", "RGPD", "platform").await;
    let agent = create_user(ctx.db()).await;
    let other = create_user(ctx.db()).await;
    enrol(ctx.db(), agent, course.id).await;
    enrol(ctx.db(), other, course.id).await;
    complete(ctx.db(), other, course.id, course.module).await;
    let req = TestRequest::post()
        .uri(&format!("/api/v1/formations/{}/rating/", course.id))
        .insert_header(("Authorization", jwt_for(other)))
        .set_json(json!({ "rating": 5 }));
    assert_eq!(status_of!(app, req), StatusCode::OK);

    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(agent)))
        .to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    let formation = &body["formations"][0];
    assert_eq!(formation["id"], course.id);
    assert_eq!(formation["category"], "RGPD");
    assert_eq!(formation["level"], Value::Null);
    assert_eq!(formation["is_mandatory"], true);
    assert_eq!(formation["deadline"], "2030-06-30");
    assert_eq!(formation["source"], "platform");
    assert_eq!(formation["instructor"], Value::Null);
    assert_eq!(formation["started_at"], Value::Null);
    assert_eq!(formation["completed_at"], Value::Null);
    assert_eq!(formation["last_activity_at"], Value::Null);
    assert_eq!(formation["last_reminder_at"], Value::Null);
    assert_eq!(formation["rating_average"], 5.0);
    assert_eq!(formation["rating_count"], 1);
    assert_eq!(formation["my_rating"], Value::Null);
    assert_eq!(formation["learners"], 2);
    let module = &formation["modules"][0];
    assert_eq!(module["sort_order"], 1);
    assert_eq!(module["duration_seconds"], 900);
    let files = module["files"].as_array().expect("files");
    assert_eq!(files.len(), 2);
    assert_eq!(files[0]["kind"], "pdf");
    assert_eq!(files[0]["title"], "guide.pdf");
    assert_eq!(files[0]["duration_seconds"], 600);
    assert_eq!(files[0]["is_required"], true);
    assert_eq!(files[0]["url"], Value::Null);
    assert_eq!(files[1]["kind"], "link");
    assert_eq!(files[1]["file_type"], "Error");
    assert_eq!(files[1]["url"], "https://example.org/texte");
    // The S3 key of a stored file is never exposed.
    assert!(!body.to_string().contains("s3/secret-key"), "{body}");

    // Once the agent progresses, the dates of their activity appear.
    complete(ctx.db(), agent, course.id, course.module).await;
    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(agent)))
        .to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    let formation = &body["formations"][0];
    assert_eq!(formation["status"], "Completed");
    assert!(formation["completed_at"].is_string(), "{formation}");
    assert!(formation["last_activity_at"].is_string(), "{formation}");
}

#[actix_web::test]
async fn rating_requires_an_enrolment_a_completion_and_a_valid_rating() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = course(&ctx, "Notation").await;
    let agent = create_user(ctx.db()).await;
    let uri = format!("/api/v1/formations/{}/rating/", course.id);
    let rate = |user: i32, rating: Value| {
        TestRequest::post()
            .uri(&uri)
            .insert_header(("Authorization", jwt_for(user)))
            .set_json(json!({ "rating": rating }))
    };

    let anonymous = TestRequest::post()
        .uri(&uri)
        .set_json(json!({ "rating": 4 }));
    assert_eq!(status_of!(app, anonymous), StatusCode::UNAUTHORIZED);
    assert_eq!(
        status_of!(app, rate(agent, json!(4))),
        StatusCode::FORBIDDEN
    );
    let unknown = TestRequest::post()
        .uri("/api/v1/formations/2147483647/rating/")
        .insert_header(("Authorization", jwt_for(agent)))
        .set_json(json!({ "rating": 4 }));
    assert_eq!(status_of!(app, unknown), StatusCode::FORBIDDEN);

    enrol(ctx.db(), agent, course.id).await;
    assert_eq!(status_of!(app, rate(agent, json!(4))), StatusCode::CONFLICT);

    complete(ctx.db(), agent, course.id, course.module).await;
    for invalid in [json!(0), json!(6), json!(-1), json!("4"), json!(null)] {
        assert_eq!(
            status_of!(app, rate(agent, invalid.clone())),
            StatusCode::BAD_REQUEST,
            "{invalid}"
        );
    }
    let path = TestRequest::post()
        .uri("/api/v1/formations/abc/rating/")
        .insert_header(("Authorization", jwt_for(agent)))
        .set_json(json!({ "rating": 4 }));
    assert_eq!(status_of!(app, path), StatusCode::BAD_REQUEST);

    let body: Value = test::call_and_read_body_json(&app, rate(agent, json!(4)).to_request()).await;
    assert_eq!(
        body,
        json!({ "rating_average": 4.0, "rating_count": 1, "my_rating": 4 })
    );

    let other = create_user(ctx.db()).await;
    enrol(ctx.db(), other, course.id).await;
    complete(ctx.db(), other, course.id, course.module).await;
    let body: Value = test::call_and_read_body_json(&app, rate(other, json!(5)).to_request()).await;
    assert_eq!(
        body,
        json!({ "rating_average": 4.5, "rating_count": 2, "my_rating": 5 })
    );

    // Rating again replaces the caller's rating.
    let body: Value = test::call_and_read_body_json(&app, rate(agent, json!(2)).to_request()).await;
    assert_eq!(
        body,
        json!({ "rating_average": 3.5, "rating_count": 2, "my_rating": 2 })
    );
}

#[actix_web::test]
async fn the_summary_counts_the_training_time_of_the_year() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let done = course(&ctx, "Temps de formation").await;
    let pending = course(&ctx, "En attente").await;
    let agent = create_user(ctx.db()).await;
    enrol(ctx.db(), agent, done.id).await;
    enrol(ctx.db(), agent, pending.id).await;
    complete(ctx.db(), agent, done.id, done.module).await;
    let summary = |query: &str| {
        TestRequest::get()
            .uri(&format!("/api/v1/formations/summary/{query}"))
            .insert_header(("Authorization", jwt_for(agent)))
    };

    let body: Value = test::call_and_read_body_json(&app, summary("").to_request()).await;
    assert_eq!(body["year"], this_year());
    assert!(body["annual_goal_minutes"].as_u64().unwrap() > 0, "{body}");
    assert_eq!(body["time_spent_seconds"], 900);
    assert_eq!(body["completed_count"], 1);
    let formations = body["formations"].as_array().expect("formations");
    assert_eq!(formations.len(), 2);
    assert_eq!(formations[0]["id"], done.id);
    assert_eq!(formations[0]["status"], "Completed");
    assert_eq!(formations[0]["time_spent_seconds"], 900);
    assert!(formations[0]["completed_at"].is_string());
    assert_eq!(formations[1]["id"], pending.id);
    assert_eq!(formations[1]["status"], "NotStarted");
    assert_eq!(formations[1]["time_spent_seconds"], 0);
    assert_eq!(formations[1]["completed_at"], Value::Null);

    let query = format!("?year={}", this_year());
    let same: Value = test::call_and_read_body_json(&app, summary(&query).to_request()).await;
    assert_eq!(same, body);

    let past: Value = test::call_and_read_body_json(&app, summary("?year=2001").to_request()).await;
    assert_eq!(past["year"], 2001);
    assert_eq!(past["time_spent_seconds"], 0);
    assert_eq!(past["completed_count"], 0);
    assert_eq!(past["formations"].as_array().unwrap().len(), 2);

    for invalid in ["?year=1999", "?year=2101", "?year=abc"] {
        assert_eq!(
            status_of!(app, summary(invalid)),
            StatusCode::BAD_REQUEST,
            "{invalid}"
        );
    }
    let anonymous = TestRequest::get().uri("/api/v1/formations/summary/");
    assert_eq!(status_of!(app, anonymous), StatusCode::UNAUTHORIZED);

    // An agent enrolled nowhere gets an empty summary.
    let newcomer = create_user(ctx.db()).await;
    let req = TestRequest::get()
        .uri("/api/v1/formations/summary/")
        .insert_header(("Authorization", jwt_for(newcomer)))
        .to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["formations"], json!([]));
    assert_eq!(body["time_spent_seconds"], 0);
}

#[actix_web::test]
async fn the_attestation_needs_a_completed_enrolment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = course(&ctx, "Attestation unitaire").await;
    set_course_meta(ctx.db(), course.id, false, "", "Sécurité", "internal").await;
    let agent = create_user(ctx.db()).await;
    let uri = format!("/api/v1/formations/{}/certificate/", course.id);
    let get = |uri: &str| {
        TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", jwt_for(agent)))
    };

    assert_eq!(
        status_of!(app, TestRequest::get().uri(&uri)),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(status_of!(app, get(&uri)), StatusCode::FORBIDDEN);
    assert_eq!(
        status_of!(app, get("/api/v1/formations/2147483647/certificate/")),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status_of!(app, get("/api/v1/formations/abc/certificate/")),
        StatusCode::BAD_REQUEST
    );
    enrol(ctx.db(), agent, course.id).await;
    assert_eq!(status_of!(app, get(&uri)), StatusCode::CONFLICT);

    complete(ctx.db(), agent, course.id, course.module).await;
    let response = test::call_service(&app, get(&uri).to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/pdf"
    );
    assert_eq!(
        response.headers().get(header::CONTENT_DISPOSITION).unwrap(),
        format!("attachment; filename=\"attestation-{}.pdf\"", course.id).as_str()
    );
    let pdf = test::read_body(response).await;
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(pdf.ends_with(b"%%EOF\n"));
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("(Agent Test) Tj"), "the fixture agent's name");
    assert!(text.contains("(Attestation unitaire) Tj"));
    assert!(text.contains("(Cat\\351gorie : S\\351curit\\351) Tj"));
    assert!(text.contains("(Dur\\351e de la formation : 15 min) Tj"));
    assert!(text.contains("/Count 1"));
}

#[actix_web::test]
async fn the_attestations_of_a_year_hold_one_page_per_completed_formation() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let first = course(&ctx, "Attestation annuelle 1").await;
    let second = course(&ctx, "Attestation annuelle 2").await;
    let pending = course(&ctx, "Attestation annuelle en cours").await;
    let agent = create_user(ctx.db()).await;
    let get = |query: &str| {
        TestRequest::get()
            .uri(&format!("/api/v1/formations/certificates/{query}"))
            .insert_header(("Authorization", jwt_for(agent)))
    };

    assert_eq!(status_of!(app, get("")), StatusCode::NOT_FOUND);
    for course in [&first, &second, &pending] {
        enrol(ctx.db(), agent, course.id).await;
    }
    complete(ctx.db(), agent, first.id, first.module).await;
    complete(ctx.db(), agent, second.id, second.module).await;

    let response = test::call_service(&app, get("").to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_DISPOSITION).unwrap(),
        format!("attachment; filename=\"attestations-{}.pdf\"", this_year()).as_str()
    );
    let pdf = test::read_body(response).await;
    let text = String::from_utf8_lossy(&pdf);
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(text.contains("/Count 2"));
    assert!(text.contains("(Attestation annuelle 1) Tj"));
    assert!(text.contains("(Attestation annuelle 2) Tj"));
    assert!(!text.contains("en cours"));

    assert_eq!(status_of!(app, get("?year=2001")), StatusCode::NOT_FOUND);
    for invalid in ["?year=1999", "?year=2101", "?year=abc"] {
        assert_eq!(
            status_of!(app, get(invalid)),
            StatusCode::BAD_REQUEST,
            "{invalid}"
        );
    }
    let anonymous = TestRequest::get().uri("/api/v1/formations/certificates/");
    assert_eq!(status_of!(app, anonymous), StatusCode::UNAUTHORIZED);
}
