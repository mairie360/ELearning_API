use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{create_attachment, create_course, create_module};
use elearning_api::database::admin::formations::get_formations::view::{
    AdminFormationRow, GetFormationsQueryView,
};

#[tokio::test]
async fn test_get_formations_without_details_has_no_modules() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;

    let view = GetFormationsQueryView::new(false, u32::MAX, 0);
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
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "Sécurité", "Bases").await;
    let module_id = create_module(&fx, course_id, "Module 1", "Contenu 1", 1).await;
    create_attachment(
        &db,
        module_id,
        "guide.pdf",
        "pdf",
        "https://s3.example.com/guide.pdf",
        1024,
    )
    .await;

    let view = GetFormationsQueryView::new(true, u32::MAX, 0);
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

/// MAIR-425: the catalogue is paginated in `id` order.
#[tokio::test]
async fn test_get_formations_pages_in_id_order() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let first = create_course(&fx, "Page 1", "Premier").await;
    let second = create_course(&fx, "Page 2", "Second").await;

    let all = db
        .fetch_all::<AdminFormationRow, _>(&GetFormationsQueryView::new(false, u32::MAX, 0))
        .await
        .expect("query failed");
    let position = all
        .iter()
        .position(|row| row.id() == first)
        .expect("course missing");
    let offset = u32::try_from(position).unwrap();

    let page = db
        .fetch_all::<AdminFormationRow, _>(&GetFormationsQueryView::new(false, 1, offset))
        .await
        .expect("query failed");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].id(), first);

    let next = db
        .fetch_all::<AdminFormationRow, _>(&GetFormationsQueryView::new(false, 1, offset + 1))
        .await
        .expect("query failed");
    assert!(next[0].id() > first && next[0].id() <= second);

    let past_the_end = db
        .fetch_all::<AdminFormationRow, _>(&GetFormationsQueryView::new(false, 1, u32::MAX))
        .await
        .expect("query failed");
    assert!(past_the_end.is_empty());
}
