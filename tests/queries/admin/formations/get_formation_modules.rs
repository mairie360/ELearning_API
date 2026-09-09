use crate::common::get_smart_db;
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::admin::formations::get_formation_modules::view::{
    AdminFormationModuleRow, GetFormationModulesQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_get_formation_modules_orders_by_sort_order() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    create_module(&db, course_id, "Second", "Contenu 2", 2).await;
    create_module(&db, course_id, "First", "Contenu 1", 1).await;

    let view = GetFormationModulesQueryView::new(course_id as u64, false);
    let rows = db
        .fetch_all::<AdminFormationModuleRow, _>(&view)
        .await
        .expect("query failed");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name(), "First");
    assert_eq!(rows[1].name(), "Second");
    assert!(rows[0].content().is_none());
}

#[tokio::test]
async fn test_get_formation_modules_with_details_includes_content() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

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

    let view = GetFormationModulesQueryView::new(course_id as u64, true);
    let rows = db
        .fetch_all::<AdminFormationModuleRow, _>(&view)
        .await
        .expect("query failed");

    let content = rows[0].content().expect("content missing");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].file_type(), "pdf");
}
