use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::create_course;
use elearning_api::database::formations::does_course_exist::view::DoesCourseExistQueryView;

#[tokio::test]
async fn test_does_course_exist_true_for_known_course() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;

    let view = DoesCourseExistQueryView::new(course_id as u64);
    let exists = db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed");

    assert!(exists);
}

#[tokio::test]
async fn test_does_course_exist_false_for_unknown_course() {
    let db = get_smart_db().await;

    let view = DoesCourseExistQueryView::new(999_999);
    let exists = db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed");

    assert!(!exists);
}
