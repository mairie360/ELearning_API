use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module, create_user, enrol};
use elearning_api::database::formations::check_module_access::view::{
    CheckModuleAccessQueryView, ModuleAccessRow,
};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

async fn access(
    db: &SmartDatabase,
    user_id: i32,
    course_id: i32,
    module_id: i32,
) -> ModuleAccessRow {
    let view = CheckModuleAccessQueryView::new(user_id as u64, course_id as u64, module_id as u64);
    db.fetch_one::<ModuleAccessRow, _>(&view)
        .await
        .expect("query failed")
}

#[tokio::test]
async fn test_check_module_access_enrolled_and_module_in_formation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let user_id = create_user(&db).await;
    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    enrol(&db, user_id, course_id).await;

    let row = access(&db, user_id, course_id, module_id).await;
    assert!(row.enrolled());
    assert!(row.module_in_formation());
}

#[tokio::test]
async fn test_check_module_access_not_enrolled() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let user_id = create_user(&db).await;
    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    // Enrolment in another course does not count.
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    enrol(&db, user_id, other_course_id).await;

    let row = access(&db, user_id, course_id, module_id).await;
    assert!(!row.enrolled());
    assert!(row.module_in_formation());
}

#[tokio::test]
async fn test_check_module_access_module_of_another_formation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let user_id = create_user(&db).await;
    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    let foreign_module_id = create_module(&db, other_course_id, "Module X", "Contenu X", 1).await;
    enrol(&db, user_id, course_id).await;

    let row = access(&db, user_id, course_id, foreign_module_id).await;
    assert!(row.enrolled());
    assert!(!row.module_in_formation());

    let row = access(&db, user_id, course_id, 999_999).await;
    assert!(row.enrolled());
    assert!(!row.module_in_formation());
}

#[tokio::test]
async fn test_check_module_access_unknown_formation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let user_id = create_user(&db).await;

    let row = access(&db, user_id, 999_999, 999_999).await;
    assert!(!row.enrolled());
    assert!(!row.module_in_formation());
}
