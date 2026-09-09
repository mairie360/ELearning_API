use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module};
use elearning_api::database::formations::complete_module::view::CompleteModuleQueryView;
use elearning_api::database::formations::get_my_formation_modules::view::{
    FormationModuleRow, GetMyFormationModulesQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_get_my_formation_modules_reflects_completion() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "Sécurité", "Bases de la sécurité").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let view = GetMyFormationModulesQueryView::new(course_id as u64, alice_id as u64);
    let rows = db
        .fetch_all::<FormationModuleRow, _>(&view)
        .await
        .expect("query failed");
    let module = rows.first().expect("module missing");
    assert_eq!(module.name(), "Module 1");
    assert!(!module.completed());

    let complete_view = CompleteModuleQueryView::new(alice_id as u64, module_id as u64);
    db.execute(complete_view)
        .await
        .expect("failed to complete module");

    let rows = db
        .fetch_all::<FormationModuleRow, _>(&view)
        .await
        .expect("query failed");
    let module = rows.first().expect("module missing");
    assert!(module.completed());
}

#[tokio::test]
async fn test_get_my_formation_modules_empty_for_unknown_course() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let view = GetMyFormationModulesQueryView::new(999_999, alice_id as u64);
    let rows = db
        .fetch_all::<FormationModuleRow, _>(&view)
        .await
        .expect("query failed");

    assert!(rows.is_empty());
}
