//! MAIR-1150, MAIR-817, MAIR-703: the administration side of the E-learning V2: indicators,
//! reminders of a mandatory formation, and creation, replacement and deletion of a formation.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;
use serde_json::{json, Value};

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::{
    archive_user, complete, count_rows, create_attachment, create_course, create_module,
    create_user, enrol, set_course_meta,
};
use crate::{init_app, status_of};

fn admin() -> String {
    jwt_for(*ADMIN_ID.get().expect("seeded admin"))
}

#[actix_web::test]
async fn the_admin_routes_of_the_v2_refuse_a_plain_agent() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent = jwt_for(create_user(ctx.db()).await);
    let course = create_course(ctx.db(), "Refus", "Refus").await;
    let body = json!({ "name": "Refused" });
    for req in [
        TestRequest::get().uri("/api/v1/admin/stats/"),
        TestRequest::post()
            .uri("/api/v1/admin/formations/")
            .set_json(body.clone()),
        TestRequest::patch()
            .uri(&format!("/api/v1/admin/formations/{course}/"))
            .set_json(body.clone()),
        TestRequest::delete().uri(&format!("/api/v1/admin/formations/{course}/")),
        TestRequest::post()
            .uri(&format!("/api/v1/admin/formations/{course}/reminders/"))
            .set_json(json!({})),
    ] {
        let req = req.insert_header(("Authorization", agent.clone()));
        assert_eq!(status_of!(app, req), StatusCode::FORBIDDEN);
    }
    // Nothing was deleted by the refused call.
    assert_eq!(count_rows(ctx.db(), "courses", course).await, 1);
    for req in [
        TestRequest::get().uri("/api/v1/admin/stats/"),
        TestRequest::delete().uri(&format!("/api/v1/admin/formations/{course}/")),
    ] {
        assert_eq!(status_of!(app, req), StatusCode::UNAUTHORIZED);
    }
}

#[actix_web::test]
async fn the_indicators_count_enrolments_progress_and_late_agents() {
    // The indicators read every row and keep the 500 latest agents: a database of its own.
    let ctx = TestContext::isolated().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "Indicateurs", "Indicateurs").await;
    let first = create_module(ctx.db(), course, "Module 1", "Contenu", 1).await;
    create_module(ctx.db(), course, "Module 2", "Contenu", 2).await;
    set_course_meta(ctx.db(), course, true, "1990-01-01", "", "internal").await;
    let late = create_user(ctx.db()).await;
    let started = create_user(ctx.db()).await;
    let archived = create_user(ctx.db()).await;
    for user in [late, started, archived] {
        enrol(ctx.db(), user, course).await;
    }
    complete(ctx.db(), started, course, first).await;
    archive_user(ctx.db(), archived).await;

    let req = TestRequest::post()
        .uri(&format!("/api/v1/admin/formations/{course}/reminders/"))
        .insert_header(("Authorization", admin()))
        .set_json(json!({ "user_ids": [late] }));
    assert_eq!(status_of!(app, req), StatusCode::OK);

    let req = TestRequest::get()
        .uri("/api/v1/admin/stats/")
        .insert_header(("Authorization", admin()))
        .to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    for key in [
        "internal_courses",
        "platform_courses",
        "mandatory_internal_courses",
        "agents",
        "participants",
        "late_mandatory",
    ] {
        assert!(body[key].is_u64(), "{key}: {body}");
    }
    assert!(body["participation_rate"].is_number());
    assert!(body["average_completion"].is_number());
    assert!(body["late_mandatory"].as_u64().unwrap() >= 2);
    assert!(!body["by_service"].as_array().unwrap().is_empty());

    // The archived agent is counted nowhere.
    let formation = body["by_formation"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == course)
        .expect("the course is listed");
    assert_eq!(formation["source"], "internal");
    assert_eq!(formation["is_mandatory"], true);
    assert_eq!(formation["enrolled"], 2);
    assert_eq!(formation["completed"], 0);
    assert_eq!(formation["average_progress"], 25.0);

    let late_agents: Vec<&Value> = body["late_agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["formation_id"] == course)
        .collect();
    assert_eq!(late_agents.len(), 2, "{late_agents:?}");
    let reminded = late_agents.iter().find(|a| a["user_id"] == late).unwrap();
    assert_eq!(reminded["first_name"], "Agent");
    assert_eq!(reminded["deadline"], "1990-01-01");
    assert_eq!(reminded["progress"], 0.0);
    assert!(reminded["last_reminder_at"].is_string());
    let progressing = late_agents
        .iter()
        .find(|a| a["user_id"] == started)
        .unwrap();
    assert_eq!(progressing["progress"], 50.0);
    assert_eq!(progressing["last_reminder_at"], Value::Null);
}

#[actix_web::test]
async fn reminders_only_reach_the_unfinished_agents_of_a_mandatory_formation() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "Rappels", "Rappels").await;
    let module = create_module(ctx.db(), course, "Module 1", "Contenu", 1).await;
    let late = create_user(ctx.db()).await;
    let finished = create_user(ctx.db()).await;
    let stranger = create_user(ctx.db()).await;
    enrol(ctx.db(), late, course).await;
    enrol(ctx.db(), finished, course).await;
    complete(ctx.db(), finished, course, module).await;
    let remind = |course: &str, body: Value| {
        TestRequest::post()
            .uri(&format!("/api/v1/admin/formations/{course}/reminders/"))
            .insert_header(("Authorization", admin()))
            .set_json(body)
    };
    let id = course.to_string();

    // Not mandatory yet: 409, nothing recorded.
    assert_eq!(
        status_of!(app, remind(&id, json!({ "user_ids": [late] }))),
        StatusCode::CONFLICT
    );
    assert_eq!(count_rows(ctx.db(), "course_reminders", course).await, 0);
    assert_eq!(
        status_of!(app, remind("2147483647", json!({}))),
        StatusCode::NOT_FOUND
    );
    for invalid in [
        json!({ "user_ids": [0] }),
        json!({ "user_ids": "all" }),
        json!({ "user_ids": [late], "message": "x" }),
    ] {
        assert_eq!(
            status_of!(app, remind(&id, invalid.clone())),
            StatusCode::BAD_REQUEST,
            "{invalid}"
        );
    }
    assert_eq!(
        status_of!(app, remind("abc", json!({}))),
        StatusCode::BAD_REQUEST
    );

    // Mandatory with a future deadline: only named agents, and only the unfinished enrolled ones.
    set_course_meta(ctx.db(), course, true, "2999-12-31", "", "internal").await;
    let req = remind(&id, json!({ "user_ids": [late, finished, stranger] })).to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body, json!({ "sent": 1 }));
    let req = remind(&id, json!({})).to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(
        body,
        json!({ "sent": 0 }),
        "nobody is late before the deadline"
    );

    // Deadline past: `{}` reminds every late agent.
    set_course_meta(ctx.db(), course, true, "2000-01-01", "", "internal").await;
    let req = remind(&id, json!({})).to_request();
    let body: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body, json!({ "sent": 1 }));
    assert_eq!(count_rows(ctx.db(), "course_reminders", course).await, 2);

    // The agent sees the latest reminder in their catalogue.
    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(late)))
        .to_request();
    let catalog: Value = test::call_and_read_body_json(&app, req).await;
    assert!(catalog["formations"][0]["last_reminder_at"].is_string());
}

fn new_course() -> Value {
    json!({
        "name": "Créée par l'API",
        "description": "Description",
        "category": "Sécurité",
        "level": "beginner",
        "is_mandatory": true,
        "deadline": "2030-01-31",
        "source": "platform",
        "instructor": "Service RH",
        "modules": [
            {
                "name": "Module A",
                "files": [
                    { "title": "Lien", "file_type": "link", "url": "https://example.org/a", "duration_seconds": 60 },
                    { "title": "Quiz", "file_type": "quiz", "is_required": false }
                ]
            },
            { "name": "Module B", "description": "Sans contenu" }
        ]
    })
}

#[actix_web::test]
async fn a_formation_is_created_with_its_modules_and_contents() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let req = TestRequest::post()
        .uri("/api/v1/admin/formations/")
        .insert_header(("Authorization", admin()))
        .set_json(new_course())
        .to_request();
    let response = test::call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = test::read_body_json(response).await;
    let id = body["id"].as_u64().expect("id");

    let learner = create_user(ctx.db()).await;
    enrol(ctx.db(), learner, i32::try_from(id).unwrap()).await;
    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(learner)))
        .to_request();
    let catalog: Value = test::call_and_read_body_json(&app, req).await;
    let formation = &catalog["formations"][0];
    assert_eq!(formation["id"], id);
    assert_eq!(formation["name"], "Créée par l'API");
    assert_eq!(formation["category"], "Sécurité");
    assert_eq!(formation["level"], "beginner");
    assert_eq!(formation["is_mandatory"], true);
    assert_eq!(formation["deadline"], "2030-01-31");
    assert_eq!(formation["source"], "platform");
    assert_eq!(formation["instructor"], "Service RH");
    let modules = formation["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0]["name"], "Module A");
    assert_eq!(modules[0]["sort_order"], 1);
    assert_eq!(modules[0]["duration_seconds"], 60);
    assert_eq!(modules[1]["name"], "Module B");
    assert_eq!(modules[1]["description"], "Sans contenu");
    assert_eq!(modules[1]["sort_order"], 2);
    let files = modules[0]["files"].as_array().unwrap();
    assert_eq!(files[0]["kind"], "link");
    assert_eq!(files[0]["url"], "https://example.org/a");
    assert_eq!(files[0]["sort_order"], 1);
    assert_eq!(files[1]["kind"], "quiz");
    assert_eq!(files[1]["is_required"], false);
    assert_eq!(files[1]["url"], Value::Null);
    assert_eq!(files[1]["sort_order"], 2);
}

#[actix_web::test]
async fn mair_703_invalid_bodies_are_refused_before_anything_is_written() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let with_file = |file: Value| {
        json!({ "name": "Refusée", "modules": [{ "name": "M", "files": [
            { "title": "Quiz", "file_type": "quiz" }, file
        ] }] })
    };
    let cases = [
        (
            with_file(json!({ "title": "L", "file_type": "link", "url": "javascript:alert(1)" })),
            "modules[0].files[1].url",
        ),
        (
            with_file(
                json!({ "title": "L", "file_type": "link", "url": "data:text/html;base64,PHNjcmlwdD4=" }),
            ),
            "modules[0].files[1].url",
        ),
        (
            with_file(json!({ "title": "L", "file_type": "link", "url": "http://example.org" })),
            "modules[0].files[1].url",
        ),
        (
            with_file(json!({ "title": "L", "file_type": "link", "url": "/relative" })),
            "modules[0].files[1].url",
        ),
        (
            with_file(json!({ "title": "L", "file_type": "link" })),
            "modules[0].files[1].url",
        ),
        (
            with_file(json!({ "title": "V", "file_type": "video" })),
            "modules[0].files[1].file_type",
        ),
        (
            with_file(json!({ "title": "Q", "file_type": "quiz", "url": "https://example.org" })),
            "modules[0].files[1].url",
        ),
        (
            with_file(json!({ "id": 3, "title": "Q", "file_type": "quiz" })),
            "modules[0].files[1].id",
        ),
        (json!({ "name": "" }), "name"),
        (json!({ "name": "x".repeat(256) }), "name"),
        (json!({ "name": "N", "level": "expert" }), "level"),
        (json!({ "name": "N", "deadline": "31/01/2030" }), "deadline"),
        (json!({ "name": "N", "source": "external" }), "source"),
        (
            json!({ "name": "N", "modules": [{ "id": 1, "name": "M" }] }),
            "modules[0].id",
        ),
        (json!({ "name": "N", "unknown": true }), "unknown field"),
    ];
    for (body, field) in cases {
        let req = TestRequest::post()
            .uri("/api/v1/admin/formations/")
            .insert_header(("Authorization", admin()))
            .set_json(body.clone())
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
        let text = String::from_utf8(test::read_body(response).await.to_vec()).unwrap();
        assert!(text.contains(field), "{body}: {text}");
    }
}

#[actix_web::test]
async fn a_formation_is_replaced_keeping_the_listed_rows() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "Avant", "Avant").await;
    let kept_module = create_module(ctx.db(), course, "Gardé", "Contenu", 1).await;
    let dropped_module = create_module(ctx.db(), course, "Supprimé", "Contenu", 2).await;
    let stored = create_attachment(
        ctx.db(),
        kept_module,
        "video.mp4",
        "video",
        "s3/video-key",
        9,
    )
    .await;
    let moved = create_attachment(ctx.db(), dropped_module, "Quiz", "quiz", "", 0).await;
    let dropped = create_attachment(ctx.db(), kept_module, "Ancien", "pdf", "s3/old-key", 9).await;
    let learner = create_user(ctx.db()).await;
    enrol(ctx.db(), learner, course).await;
    complete(ctx.db(), learner, course, kept_module).await;
    let other_course = create_course(ctx.db(), "Autre", "Autre").await;
    let other_module = create_module(ctx.db(), other_course, "Autre", "Contenu", 1).await;
    let foreign = create_attachment(ctx.db(), other_module, "Autre", "quiz", "", 0).await;
    let uri = format!("/api/v1/admin/formations/{course}/");
    let patch = |uri: &str, body: Value| {
        TestRequest::patch()
            .uri(uri)
            .insert_header(("Authorization", admin()))
            .set_json(body)
    };
    let body = |files: Value| {
        json!({
            "name": "Après",
            "is_mandatory": false,
            "modules": [
                { "name": "Nouveau", "files": [] },
                { "id": kept_module, "name": "Gardé et renommé", "files": files }
            ]
        })
    };

    // Ids of another formation, a stored file whose type changes: 400, nothing written.
    for files in [
        json!([{ "id": foreign, "title": "X", "file_type": "quiz" }]),
        json!([{ "id": stored, "title": "X", "file_type": "pdf" }]),
        json!([{ "id": moved, "title": "X", "file_type": "video" }]),
    ] {
        assert_eq!(
            status_of!(app, patch(&uri, body(files.clone()))),
            StatusCode::BAD_REQUEST,
            "{files}"
        );
    }
    let foreign_module = json!({ "name": "N", "modules": [{ "id": other_module, "name": "M" }] });
    assert_eq!(
        status_of!(app, patch(&uri, foreign_module)),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(count_rows(ctx.db(), "course_modules", course).await, 2);
    assert_eq!(count_rows(ctx.db(), "course_attachments", course).await, 3);
    assert_eq!(
        status_of!(
            app,
            patch("/api/v1/admin/formations/2147483647/", body(json!([])))
        ),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status_of!(app, patch("/api/v1/admin/formations/abc/", body(json!([])))),
        StatusCode::BAD_REQUEST
    );

    let files = json!([
        { "title": "Nouveau lien", "file_type": "link", "url": "https://example.org/b" },
        { "id": stored, "title": "Vidéo renommée", "file_type": "video", "duration_seconds": 120 },
        { "id": moved, "title": "Quiz déplacé", "file_type": "link", "url": "https://example.org/q" }
    ]);
    let req = patch(&uri, body(files)).to_request();
    let response: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(response, json!({ "id": course }));

    assert_eq!(count_rows(ctx.db(), "course_modules", course).await, 2);
    assert_eq!(count_rows(ctx.db(), "course_attachments", course).await, 3);
    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(learner)))
        .to_request();
    let catalog: Value = test::call_and_read_body_json(&app, req).await;
    let formation = catalog["formations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == course)
        .unwrap();
    assert_eq!(formation["name"], "Après");
    assert_eq!(formation["is_mandatory"], false);
    let modules = formation["modules"].as_array().unwrap();
    assert_eq!(modules[0]["name"], "Nouveau");
    assert_eq!(modules[1]["id"], kept_module);
    assert_eq!(modules[1]["name"], "Gardé et renommé");
    // The progress on a kept module is kept.
    assert_eq!(modules[1]["completed"], true);
    let files = modules[1]["files"].as_array().unwrap();
    assert_eq!(files[1]["id"], stored);
    assert_eq!(files[1]["title"], "Vidéo renommée");
    assert_eq!(files[1]["file_name"], "video.mp4");
    assert_eq!(files[1]["duration_seconds"], 120);
    assert_eq!(files[2]["id"], moved);
    assert_eq!(files[2]["url"], "https://example.org/q");
    assert!(!files.iter().any(|f| f["id"] == dropped));

    // The stored file kept its object key: its signed URL is still issued for it.
    let req = TestRequest::get()
        .uri(&format!(
            "/api/v1/formations/{course}/{kept_module}/{stored}/"
        ))
        .insert_header(("Authorization", jwt_for(learner)));
    assert_eq!(status_of!(app, req), StatusCode::OK);
    assert!(ctx
        .storage
        .calls
        .lock()
        .unwrap()
        .contains(&"s3/video-key".to_string()));
}

#[actix_web::test]
async fn a_formation_is_deleted_with_everything_it_holds() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let course = create_course(ctx.db(), "À supprimer", "À supprimer").await;
    let module = create_module(ctx.db(), course, "Module", "Contenu", 1).await;
    create_attachment(ctx.db(), module, "guide.pdf", "pdf", "s3/key", 1).await;
    let learner = create_user(ctx.db()).await;
    enrol(ctx.db(), learner, course).await;
    let delete = |uri: String| {
        TestRequest::delete()
            .uri(&uri)
            .insert_header(("Authorization", admin()))
    };
    let uri = format!("/api/v1/admin/formations/{course}/");

    assert_eq!(status_of!(app, delete(uri.clone())), StatusCode::NO_CONTENT);
    assert_eq!(count_rows(ctx.db(), "courses", course).await, 0);
    assert_eq!(count_rows(ctx.db(), "course_modules", course).await, 0);
    assert_eq!(status_of!(app, delete(uri)), StatusCode::NOT_FOUND);
    assert_eq!(
        status_of!(app, delete("/api/v1/admin/formations/abc/".to_string())),
        StatusCode::BAD_REQUEST
    );
    let req = TestRequest::get()
        .uri("/api/v1/formations/catalog/")
        .insert_header(("Authorization", jwt_for(learner)))
        .to_request();
    let catalog: Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(catalog["formations"], json!([]));
}
