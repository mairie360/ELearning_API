use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use elearning_api::database::formations::complete_module::view::{
    CompleteModuleQueryView, CompleteModuleRow,
};
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

async fn complete(
    db: &SmartDatabase,
    user_id: i32,
    course_id: i32,
    module_id: i32,
) -> Result<CompleteModuleRow, ApiLibError> {
    db.fetch_one(&CompleteModuleQueryView::new(
        user_id as u64,
        course_id as u64,
        module_id as u64,
    ))
    .await
}

#[tokio::test]
async fn test_complete_module_success_and_idempotent() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let row = complete(&db, user_id, course_id, module_id)
        .await
        .expect("query failed");
    assert!(row.completed());

    // Completing the same module twice must not error (upsert).
    let row = complete(&db, user_id, course_id, module_id)
        .await
        .expect("query failed");
    assert!(row.completed());
}

#[tokio::test]
async fn test_complete_module_requires_previous_modules() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;
    // Created out of order: `sort_order` decides, not the id.
    let second = create_module(&db, course_id, "Module 2", "Contenu 2", 2).await;
    let first = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let row = complete(&db, user_id, course_id, second)
        .await
        .expect("query failed");
    assert!(!row.completed(), "module 2 must wait for module 1");

    let row = complete(&db, user_id, course_id, first)
        .await
        .expect("query failed");
    assert!(row.completed());
    let row = complete(&db, user_id, course_id, second)
        .await
        .expect("query failed");
    assert!(row.completed());
}

#[tokio::test]
async fn test_complete_module_breaks_sort_order_ties_by_id() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;
    let first = create_module(&db, course_id, "Module A", "Contenu A", 1).await;
    let second = create_module(&db, course_id, "Module B", "Contenu B", 1).await;

    let row = complete(&db, user_id, course_id, second)
        .await
        .expect("query failed");
    assert!(!row.completed());
    let row = complete(&db, user_id, course_id, first)
        .await
        .expect("query failed");
    assert!(row.completed());
}

#[tokio::test]
async fn test_complete_module_ignores_other_users_and_courses() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;
    let other_user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;
    let first = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    let second = create_module(&db, course_id, "Module 2", "Contenu 2", 2).await;
    // The first module of another course is never blocked by this one.
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    let other_first = create_module(&db, other_course_id, "Module 1", "Contenu 1", 1).await;
    enrol(&db, user_id, other_course_id).await;
    enrol(&db, other_user_id, course_id).await;
    let row = complete(&db, user_id, other_course_id, other_first)
        .await
        .expect("query failed");
    assert!(row.completed());

    // Another user's progress does not unlock the next module.
    complete(&db, other_user_id, course_id, first)
        .await
        .expect("query failed");
    let row = complete(&db, user_id, course_id, second)
        .await
        .expect("query failed");
    assert!(!row.completed());
}

#[tokio::test]
async fn test_complete_module_requires_enrolment() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let row = complete(&db, user_id, course_id, module_id)
        .await
        .expect("query failed");
    assert!(!row.enrolled());
    assert!(!row.completed());

    // Nothing was written: once enrolled, the module is still to do.
    enrol(&db, user_id, course_id).await;
    let row = complete(&db, user_id, course_id, module_id)
        .await
        .expect("query failed");
    assert!(row.enrolled() && row.completed());
}

#[tokio::test]
async fn test_complete_module_outside_the_course_writes_nothing() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let user_id = create_user(&db).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    enrol(&db, user_id, course_id).await;
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    let other_module = create_module(&db, other_course_id, "Module 1", "Contenu 1", 1).await;

    for module_id in [other_module, 999_999] {
        let row = complete(&db, user_id, course_id, module_id)
            .await
            .expect("query failed");
        assert!(row.enrolled());
        assert!(!row.module_in_formation());
        assert!(!row.completed());
    }
}
