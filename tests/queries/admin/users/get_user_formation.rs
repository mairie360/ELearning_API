use crate::common::get_smart_db;
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::admin::users::get_user_formation::view::{
    GetUserFormationQueryView, UserFormationModuleRow,
};
use elearning_api::database::formations::complete_module::view::CompleteModuleQueryView;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_get_user_formation_content_is_always_an_array() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    create_attachment(
        &db,
        module_id,
        "guide.pdf",
        "pdf",
        "https://s3.example.com/guide.pdf",
        1024,
    )
    .await;

    let view = GetUserFormationQueryView::new(course_id as u64, alice_id as u64, false);
    let rows = db
        .fetch_all::<UserFormationModuleRow, _>(&view)
        .await
        .expect("query failed");
    // `details=false`: content must be an empty array, not null/missing.
    assert!(rows[0].content().is_empty());
    assert!(!rows[0].is_completed());

    let complete_view = CompleteModuleQueryView::new(alice_id as u64, module_id as u64);
    db.execute(complete_view)
        .await
        .expect("failed to complete module");

    let view = GetUserFormationQueryView::new(course_id as u64, alice_id as u64, true);
    let rows = db
        .fetch_all::<UserFormationModuleRow, _>(&view)
        .await
        .expect("query failed");
    assert_eq!(rows[0].content().len(), 1);
    assert!(rows[0].is_completed());
}
