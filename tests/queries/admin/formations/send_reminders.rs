use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{
    archive_user, complete, count_rows, create_course, create_module, create_user, enrol,
    set_course_meta,
};
use elearning_api::database::admin::formations::send_reminders::view::{
    SendRemindersQueryView, SendRemindersRow,
};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

#[tokio::test]
async fn test_send_reminders_targets_unfinished_agents_of_a_mandatory_course() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let admin = u64::try_from(*ADMIN_ID.get().expect("seeded admin")).unwrap();
    let course = create_course(&fx, "Rappels", "Rappels").await;
    let module = create_module(&fx, course, "Module", "Contenu", 1).await;
    let (late, finished, archived, stranger) = (
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
    );
    for user in [late, finished, archived] {
        enrol(&fx, user, course).await;
    }
    complete(&fx, finished, course, module).await;
    archive_user(&fx, archived).await;
    let send =
        |ids: Option<Vec<u64>>| SendRemindersQueryView::new(course as u64, ids.as_deref(), admin);
    let all: Vec<u64> = [late, finished, archived, stranger]
        .iter()
        .map(|id| *id as u64)
        .collect();

    let row: SendRemindersRow = db.fetch_one(&send(Some(all.clone()))).await.unwrap();
    assert!(row.course_exists());
    assert!(!row.mandatory());
    assert_eq!(row.sent(), 0);

    set_course_meta(&fx, course, true, "2999-01-01", "", "internal").await;
    let row: SendRemindersRow = db.fetch_one(&send(Some(all))).await.unwrap();
    assert!(row.mandatory());
    assert_eq!(
        row.sent(),
        1,
        "only the enrolled, active and unfinished agent"
    );
    let row: SendRemindersRow = db.fetch_one(&send(None)).await.unwrap();
    assert_eq!(row.sent(), 0, "the deadline is not past");

    set_course_meta(&fx, course, true, "2001-01-01", "", "internal").await;
    let row: SendRemindersRow = db.fetch_one(&send(None)).await.unwrap();
    assert_eq!(row.sent(), 1);
    assert_eq!(count_rows(&fx, "course_reminders", course).await, 2);

    let row: SendRemindersRow = db
        .fetch_one(&SendRemindersQueryView::new(
            u64::from(u32::MAX),
            None,
            admin,
        ))
        .await
        .unwrap();
    assert!(!row.course_exists());
}
