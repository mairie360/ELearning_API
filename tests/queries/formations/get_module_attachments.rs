use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::formations::get_module_attachments::view::{
    GetModuleAttachmentsQueryView, ModuleAttachmentRow,
};

#[tokio::test]
async fn test_get_module_attachments_lists_files() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;
    create_attachment(
        &fx,
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
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&fx, "Autre formation", "Autre").await;
    let module_id = create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;
    create_attachment(
        &fx,
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
