use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use elearning_api::database::admin::users::unsub_user_formation::view::{
    UnsubUserFormationQueryView, UnsubUserFormationRow,
};
use elearning_api::database::formations::complete_module::view::{
    CompleteModuleQueryView, CompleteModuleRow,
};
use elearning_api::database::formations::get_my_formation_modules::view::{
    FormationModuleRow, GetMyFormationModulesQueryView,
};
use elearning_api::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

async fn unsub(db: &SmartDatabase, user_id: i32, course_id: i32) -> UnsubUserFormationRow {
    db.fetch_one(&UnsubUserFormationQueryView::new(
        user_id as u64,
        course_id as u64,
    ))
    .await
    .expect("unsub failed")
}

async fn complete(db: &SmartDatabase, user_id: i32, module_id: i32) {
    let row: CompleteModuleRow = db
        .fetch_one(&CompleteModuleQueryView::new(
            user_id as u64,
            module_id as u64,
        ))
        .await
        .expect("completion failed");
    assert!(row.completed());
}

async fn completed_modules(db: &SmartDatabase, user_id: i32, course_id: i32) -> usize {
    let view = GetMyFormationModulesQueryView::new(course_id as u64, user_id as u64);
    db.fetch_all::<FormationModuleRow, _>(&view)
        .await
        .expect("query failed")
        .iter()
        .filter(|row| row.completed())
        .count()
}

#[tokio::test]
async fn test_unsub_user_formation_removes_registration() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;

    assert!(unsub(&db, user_id, course_id).await.unregistered());

    let list_view = GetMyFormationsQueryView::new(user_id as u64);
    let rows = db
        .fetch_all::<FormationSummaryRow, _>(&list_view)
        .await
        .expect("query failed");
    assert!(!rows.iter().any(|row| row.id() == course_id));
}

#[tokio::test]
async fn test_unsub_user_formation_deletes_progress_of_that_course_only() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    let other_module_id = create_module(&db, other_course_id, "Module 1", "Contenu 1", 1).await;
    enrol(&db, user_id, course_id).await;
    enrol(&db, user_id, other_course_id).await;
    complete(&db, user_id, module_id).await;
    complete(&db, user_id, other_module_id).await;

    assert!(unsub(&db, user_id, course_id).await.unregistered());

    // Enrolling again starts from scratch...
    enrol(&db, user_id, course_id).await;
    assert_eq!(completed_modules(&db, user_id, course_id).await, 0);
    // ... while the other course keeps its progress.
    assert_eq!(completed_modules(&db, user_id, other_course_id).await, 1);
}

#[tokio::test]
async fn test_unsub_user_formation_reports_when_not_registered() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    assert!(!unsub(&db, user_id, course_id).await.unregistered());
    assert!(!unsub(&db, user_id, 999_999).await.unregistered());
}
