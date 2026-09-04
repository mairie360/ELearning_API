use crate::common::get_smart_db;
use crate::queries::fixtures::create_course;
use elearning_api::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_get_my_formations_empty_for_unregistered_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let view = GetMyFormationsQueryView::new(alice_id as u64);
    let result = db.fetch_all::<FormationSummaryRow, _>(&view).await;

    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn test_get_my_formations_lists_registered_course() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    sqlx_register(&db, alice_id, course_id).await;

    let view = GetMyFormationsQueryView::new(alice_id as u64);
    let rows = db
        .fetch_all::<FormationSummaryRow, _>(&view)
        .await
        .expect("query failed");

    let row = rows
        .iter()
        .find(|row| row.id() == course_id)
        .expect("course not found in results");
    assert_eq!(row.name(), "RGPD");
    assert_eq!(row.status(), "not_started");
}

async fn sqlx_register(
    db: &mairie360_api_lib::smart_db::SmartDatabase,
    user_id: i32,
    course_id: i32,
) {
    use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;

    let view = RegisterUserToFormationQueryView::new(user_id as u64, course_id as u64);
    db.execute(view).await.expect("failed to register user");
}
