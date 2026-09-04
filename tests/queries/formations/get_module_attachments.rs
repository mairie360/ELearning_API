use crate::common::get_smart_db;
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::formations::get_module_attachments::view::{
    GetModuleAttachmentsQueryView, ModuleAttachmentRow,
};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_get_module_attachments_lists_files() {
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

    let view = GetModuleAttachmentsQueryView::new(course_id as u64, module_id as u64);
    let rows = db
        .fetch_all::<ModuleAttachmentRow, _>(&view)
        .await
        .expect("query failed");

    let file = rows.first().expect("attachment missing");
    assert_eq!(file.file_name(), "guide.pdf");
    assert_eq!(file.file_type(), "pdf");
    assert_eq!(file.file_size_bytes(), Some(1024));
}

#[tokio::test]
async fn test_get_module_attachments_scoped_to_formation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&db, "Autre formation", "Autre").await;
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

    // A module_id that belongs to a different course must yield no rows.
    let view = GetModuleAttachmentsQueryView::new(other_course_id as u64, module_id as u64);
    let rows = db
        .fetch_all::<ModuleAttachmentRow, _>(&view)
        .await
        .expect("query failed");

    assert!(rows.is_empty());
}
