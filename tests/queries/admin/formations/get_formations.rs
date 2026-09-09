use crate::common::get_smart_db;
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::admin::formations::get_formations::view::{
    AdminFormationRow, GetFormationsQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_get_formations_without_details_has_no_modules() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    let view = GetFormationsQueryView::new(false);
    let rows = db
        .fetch_all::<AdminFormationRow, _>(&view)
        .await
        .expect("query failed");

    let course = rows
        .iter()
        .find(|row| row.id() == course_id)
        .expect("course missing");
    assert!(course.modules().is_none());
}

#[tokio::test]
async fn test_get_formations_with_details_nests_modules_and_content() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "Sécurité", "Bases").await;
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

    let view = GetFormationsQueryView::new(true);
    let rows = db
        .fetch_all::<AdminFormationRow, _>(&view)
        .await
        .expect("query failed");

    let course = rows
        .iter()
        .find(|row| row.id() == course_id)
        .expect("course missing");
    let modules = course.modules().expect("modules missing");
    let module = modules
        .iter()
        .find(|module| module.id() == module_id)
        .expect("module missing");
    let content = module.content().expect("content missing");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].file_name(), "guide.pdf");
}
