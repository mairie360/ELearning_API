use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{create_course, create_user, enrol};
use elearning_api::database::formations::is_enrolled::view::IsEnrolledQueryView;

#[tokio::test]
async fn test_is_enrolled_follows_user_courses() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;

    let user_id = create_user(&fx).await;
    let course_id = create_course(&fx, "RGPD", "Comprendre le RGPD").await;
    let other_course_id = create_course(&fx, "Autre", "Autre formation").await;
    enrol(&fx, user_id, other_course_id).await;

    let view = IsEnrolledQueryView::new(user_id as u64, course_id as u64);
    assert!(!db
        .fetch_scalar::<bool, _>(&view)
        .await
        .expect("query failed"));

    enrol(&fx, user_id, course_id).await;
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
