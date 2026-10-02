use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_user};
use elearning_api::database::admin::formations::register_user_to_formation::view::{
    RegisterUserToFormationQueryView, RegisterUserToFormationRow,
};
use elearning_api::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

async fn register(db: &SmartDatabase, user_id: i32, course_id: i32) -> RegisterUserToFormationRow {
    db.fetch_one(&RegisterUserToFormationQueryView::new(
        user_id as u64,
        course_id as u64,
    ))
    .await
    .expect("registration failed")
}

async fn registrations(db: &SmartDatabase, user_id: i32, course_id: i32) -> usize {
    let list_view = GetMyFormationsQueryView::new(user_id as u64);
    db.fetch_all::<FormationSummaryRow, _>(&list_view)
        .await
        .expect("query failed")
        .iter()
        .filter(|row| row.id() == course_id)
        .count()
}

#[tokio::test]
async fn test_register_user_to_formation_success_and_idempotent() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    let row = register(&db, user_id, course_id).await;
    assert!(row.user_exists() && row.course_exists());

    // Registering the same pair twice must be a no-op, not an error.
    let row = register(&db, user_id, course_id).await;
    assert!(row.user_exists() && row.course_exists());

    assert_eq!(
        registrations(&db, user_id, course_id).await,
        1,
        "registering twice must not duplicate the row"
    );
}

#[tokio::test]
async fn test_register_user_to_formation_reports_unknown_user_or_course() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;
    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    let row = register(&db, 999_999, course_id).await;
    assert!(!row.user_exists());
    assert!(row.course_exists());

    let row = register(&db, user_id, 999_999).await;
    assert!(row.user_exists());
    assert!(!row.course_exists());
    assert_eq!(registrations(&db, user_id, 999_999).await, 0);
}
