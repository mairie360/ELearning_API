use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use elearning_api::database::admin::users::purge_user_formation_progress::view::PurgeUserFormationProgressQueryView;
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

/// Runs both statements of the unenrolment in one transaction, like the
/// `DELETE /admin/users/{user_id}/{formation_id}/` handler.
async fn unsub(db: &SmartDatabase, user_id: i32, course_id: i32) -> UnsubUserFormationRow {
    let mut tx = db.begin().await.expect("begin failed");
    let row: UnsubUserFormationRow = tx
        .fetch_one(&UnsubUserFormationQueryView::new(
            user_id as u64,
            course_id as u64,
        ))
        .await
        .expect("unsub failed");
    if row.unregistered() {
        tx.execute(&PurgeUserFormationProgressQueryView::new(
            user_id as u64,
            course_id as u64,
        ))
        .await
        .expect("purge failed");
    }
    tx.commit().await.expect("commit failed");
    row
}

async fn complete(db: &SmartDatabase, user_id: i32, course_id: i32, module_id: i32) {
    let row: CompleteModuleRow = db
        .fetch_one(&CompleteModuleQueryView::new(
            user_id as u64,
            course_id as u64,
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
    complete(&db, user_id, course_id, module_id).await;
    complete(&db, user_id, other_course_id, other_module_id).await;

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

/// MAIR-420: a completion still in flight when the unenrolment starts must not
/// survive it. The completion holds the enrolment row `FOR SHARE`, so the
/// unenrolment waits for it, then purges the progress it wrote.
#[tokio::test]
async fn test_unsub_user_formation_purges_a_completion_in_flight() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    enrol(&db, user_id, course_id).await;

    let mut completion = db.begin().await.expect("begin failed");
    let row: CompleteModuleRow = completion
        .fetch_one(&CompleteModuleQueryView::new(
            user_id as u64,
            course_id as u64,
            module_id as u64,
        ))
        .await
        .expect("completion failed");
    assert!(row.completed());

    let unsub_db = db.clone();
    let unsubscription =
        tokio::spawn(async move { unsub(&unsub_db, user_id, course_id).await.unregistered() });
    // Give the unenrolment time to block on the enrolment row.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(!unsubscription.is_finished(), "the unenrolment must wait");

    completion.commit().await.expect("commit failed");
    assert!(unsubscription.await.expect("unsub task panicked"));

    enrol(&db, user_id, course_id).await;
    assert_eq!(completed_modules(&db, user_id, course_id).await, 0);
}

/// MAIR-420: once the unenrolment is committed, a completion finds no
/// enrolment and writes nothing.
#[tokio::test]
async fn test_complete_after_unsub_writes_nothing() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    enrol(&db, user_id, course_id).await;
    assert!(unsub(&db, user_id, course_id).await.unregistered());

    let row: CompleteModuleRow = db
        .fetch_one(&CompleteModuleQueryView::new(
            user_id as u64,
            course_id as u64,
            module_id as u64,
        ))
        .await
        .expect("completion failed");
    assert!(!row.enrolled());

    enrol(&db, user_id, course_id).await;
    assert_eq!(completed_modules(&db, user_id, course_id).await, 0);
}
