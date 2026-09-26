use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_user, enrol};
use elearning_api::database::formations::is_enrolled::view::IsEnrolledQueryView;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[tokio::test]
async fn test_is_enrolled_follows_user_courses() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let user_id = create_user(&db).await;
    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&db, "Autre", "Autre formation").await;
    enrol(&db, user_id, other_course_id).await;

    let view = IsEnrolledQueryView::new(user_id as u64, course_id as u64);
    assert!(!db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed"));

    enrol(&db, user_id, course_id).await;
    assert!(db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed"));

    let view = IsEnrolledQueryView::new(user_id as u64, 999_999);
    assert!(!db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed"));
}
