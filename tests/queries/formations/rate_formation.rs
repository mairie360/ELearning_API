use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{complete, create_course, create_module, create_user, enrol};
use elearning_api::database::formations::rate_formation::view::{
    RateFormationQueryView, RateFormationRow,
};
use mairie360_api_lib::smart_db::SmartDatabase;

async fn rate(db: &SmartDatabase, user: i32, course: i32, rating: u8) -> RateFormationRow {
    db.fetch_one(&RateFormationQueryView::new(
        user as u64,
        course as u64,
        rating,
    ))
    .await
    .expect("query failed")
}

#[tokio::test]
async fn test_rate_formation_needs_an_enrolment_and_a_completion() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Notation", "Notation").await;
    let module = create_module(&fx, course, "Module", "Contenu", 1).await;
    let user = create_user(&fx).await;

    let row = rate(&db, user, course, 4).await;
    assert!(!row.enrolled());
    assert!(!row.completed());
    assert_eq!(row.my_rating(), None);

    enrol(&fx, user, course).await;
    let row = rate(&db, user, course, 4).await;
    assert!(row.enrolled());
    assert!(!row.completed());
    assert_eq!(row.rating_count(), 0, "nothing was written");

    complete(&fx, user, course, module).await;
    let row = rate(&db, user, course, 4).await;
    assert!(row.completed());
    assert_eq!(row.my_rating(), Some(4));
    assert_eq!(row.rating_count(), 1);
    assert_eq!(row.rating_average(), Some(4.0));
}

#[tokio::test]
async fn test_rate_formation_replaces_the_rating_and_averages_every_learner() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Moyenne", "Moyenne").await;
    let module = create_module(&fx, course, "Module", "Contenu", 1).await;
    let (first, second, third) = (
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
    );
    for user in [first, second, third] {
        enrol(&fx, user, course).await;
        complete(&fx, user, course, module).await;
    }
    rate(&db, first, course, 5).await;
    rate(&db, second, course, 4).await;
    let row = rate(&db, third, course, 4).await;
    assert_eq!(row.rating_count(), 3);
    assert_eq!(row.rating_average(), Some(4.3));

    let row = rate(&db, first, course, 1).await;
    assert_eq!(
        row.rating_count(),
        3,
        "a second rating replaces the first one"
    );
    assert_eq!(row.rating_average(), Some(3.0));
    assert_eq!(row.my_rating(), Some(1));
}
