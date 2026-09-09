use crate::common::get_smart_db;
use crate::queries::fixtures::create_course;
use elearning_api::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_does_course_exist_true_for_known_course() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    let view = DoesCourseExistQueryView::new(course_id as u64);
    let exists = db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed");

    assert!(exists);
}

#[tokio::test]
async fn test_does_course_exist_false_for_unknown_course() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = DoesCourseExistQueryView::new(999_999);
    let exists = db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed");

    assert!(!exists);
}
