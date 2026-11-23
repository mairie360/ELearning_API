use chrono::Datelike;

use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{
    complete, create_attachment, create_course, create_module, create_user, enrol,
    set_attachment_duration,
};
use elearning_api::database::formations::get_my_summary::view::{
    GetMySummaryQueryView, SummaryRow,
};

#[tokio::test]
async fn test_get_my_summary_sums_the_modules_completed_in_the_year() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Résumé", "Résumé").await;
    let first = create_module(&fx, course, "Module 1", "Contenu", 1).await;
    let second = create_module(&fx, course, "Module 2", "Contenu", 2).await;
    let a = create_attachment(&fx, first, "a.pdf", "pdf", "s3/a", 1).await;
    let b = create_attachment(&fx, first, "b.pdf", "pdf", "s3/b", 1).await;
    let c = create_attachment(&fx, second, "c.pdf", "pdf", "s3/c", 1).await;
    set_attachment_duration(&fx, a, 600).await;
    set_attachment_duration(&fx, b, 120).await;
    set_attachment_duration(&fx, c, 3600).await;
    let user = create_user(&fx).await;
    enrol(&fx, user, course).await;
    complete(&fx, user, course, first).await;
    let year = chrono::Utc::now().year();

    let row: SummaryRow = db
        .fetch_one(&GetMySummaryQueryView::new(user as u64, year))
        .await
        .expect("query failed");
    assert!(row.annual_goal_minutes().is_some_and(|goal| goal > 0));
    assert_eq!(
        row.time_spent_seconds(),
        720,
        "only the completed module counts"
    );
    assert_eq!(row.completed_count(), 0);
    assert_eq!(row.formations().len(), 1);
    assert_eq!(row.formations()[0].id(), course);
    assert_eq!(row.formations()[0].status(), "in_progress");
    assert_eq!(row.formations()[0].time_spent_seconds(), 720);

    complete(&fx, user, course, second).await;
    let row: SummaryRow = db
        .fetch_one(&GetMySummaryQueryView::new(user as u64, year))
        .await
        .expect("query failed");
    assert_eq!(row.time_spent_seconds(), 4320);
    assert_eq!(row.completed_count(), 1);
    assert!(row.formations()[0].completed_at().is_some());

    let row: SummaryRow = db
        .fetch_one(&GetMySummaryQueryView::new(user as u64, 2001))
        .await
        .expect("query failed");
    assert_eq!(row.time_spent_seconds(), 0);
    assert_eq!(row.completed_count(), 0);
    assert_eq!(row.formations()[0].time_spent_seconds(), 0);
}

#[tokio::test]
async fn test_get_my_summary_of_an_agent_enrolled_nowhere() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let user = create_user(&fx).await;
    let row: SummaryRow = db
        .fetch_one(&GetMySummaryQueryView::new(user as u64, 2026))
        .await
        .expect("query failed");
    assert_eq!(row.time_spent_seconds(), 0);
    assert!(row.formations().is_empty());
}
