use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::formations::get_attachment::view::{
    AttachmentRow, GetAttachmentQueryView,
};

#[tokio::test]
async fn test_get_attachment_returns_scoped_row() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;
    let attachment_id = create_attachment(
        &fx,
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
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&fx, "Autre", "Autre").await;
    let module_id = create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;
    let attachment_id = create_attachment(
        &fx,
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
