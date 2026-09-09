use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module};
use elearning_api::database::formations::complete_module::view::CompleteModuleQueryView;
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_complete_module_success_and_idempotent() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let view = CompleteModuleQueryView::new(alice_id as u64, module_id as u64);
    let result = db.execute(view).await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);

    // Completing the same module twice must not error (upsert).
    let view = CompleteModuleQueryView::new(alice_id as u64, module_id as u64);
    let result = db.execute(view).await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn test_complete_module_unknown_module_is_foreign_key_violation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let view = CompleteModuleQueryView::new(alice_id as u64, 999_999);
    let result = db.execute(view).await;

    assert!(matches!(
        result,
        Err(ApiLibError::Database(DbError::ForeignKeyViolation(_)))
    ));
}
