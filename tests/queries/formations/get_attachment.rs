use crate::common::get_smart_db;
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::formations::get_attachment::view::{
    AttachmentRow, GetAttachmentQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_get_attachment_returns_scoped_row() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    let attachment_id = create_attachment(
        &db,
        module_id,
        "guide.pdf",
        "pdf",
        "courses/1/guide.pdf",
        1024,
    )
    .await;

    let view =
        GetAttachmentQueryView::new(course_id as u64, module_id as u64, attachment_id as u64);
    let rows = db
        .fetch_all::<AttachmentRow, _>(&view)
        .await
        .expect("query failed");

    let row = rows.first().expect("attachment missing");
    assert_eq!(row.id(), attachment_id);
    assert_eq!(row.file_name(), "guide.pdf");
    assert_eq!(row.file_type(), "pdf");
    assert_eq!(row.file_url(), "courses/1/guide.pdf");
}

#[tokio::test]
async fn test_get_attachment_rejects_mismatched_triple() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&db, "Autre", "Autre").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;
    let attachment_id = create_attachment(
        &db,
        module_id,
        "guide.pdf",
        "pdf",
        "courses/1/guide.pdf",
        1024,
    )
    .await;

    // Right attachment + module, wrong course.
    let view = GetAttachmentQueryView::new(
        other_course_id as u64,
        module_id as u64,
        attachment_id as u64,
    );
    let rows = db
        .fetch_all::<AttachmentRow, _>(&view)
        .await
        .expect("query failed");
    assert!(rows.is_empty());

    // Right course + module, unknown attachment id.
    let view = GetAttachmentQueryView::new(course_id as u64, module_id as u64, 999_999);
    let rows = db
        .fetch_all::<AttachmentRow, _>(&view)
        .await
        .expect("query failed");
    assert!(rows.is_empty());
}
