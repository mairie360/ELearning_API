//! MAIR-817: the learners of a formation, one page in one statement.

use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{
    archive_user, complete, create_course, create_module, create_named_user, enrol, set_course_meta,
};
use elearning_api::database::admin::formations::get_formation_learners::view::{
    FormationLearnersRow, GetFormationLearnersQueryView,
};
use elearning_api::database::admin::formations::send_reminders::view::{
    SendRemindersQueryView, SendRemindersRow,
};
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

#[tokio::test]
async fn test_learners_are_paged_by_name_with_their_progress() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Apprenants", "Apprenants").await;
    let first = create_module(&fx, course, "Module 1", "Contenu", 1).await;
    create_module(&fx, course, "Module 2", "Contenu", 2).await;
    let zola = create_named_user(&fx, "Emile", "Zola").await;
    let blum_b = create_named_user(&fx, "Bernard", "Blum").await;
    let blum_a = create_named_user(&fx, "Anne", "Blum").await;
    let archived = create_named_user(&fx, "Aaron", "Aaron").await;
    let stranger = create_named_user(&fx, "Abel", "Abel").await;
    for user in [zola, blum_b, blum_a, archived] {
        enrol(&fx, user, course).await;
    }
    complete(&fx, zola, course, first).await;
    archive_user(&fx, archived).await;
    let admin = u64::try_from(*ADMIN_ID.get().expect("seeded admin")).unwrap();
    set_course_meta(&fx, course, true, "2999-01-01", "", "internal").await;
    let reminded: SendRemindersRow = db
        .fetch_one(&SendRemindersQueryView::new(
            course as u64,
            Some(&[blum_a as u64]),
            admin,
        ))
        .await
        .unwrap();
    assert_eq!(reminded.sent(), 1);

    let row: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(course as u64, 50, 0))
        .await
        .unwrap();
    assert!(row.course_exists());
    assert_eq!(row.total(), 3, "the archived agent is left out");
    let ids: Vec<i32> = row.learners().iter().map(|l| l.user_id()).collect();
    assert_eq!(ids, vec![blum_a, blum_b, zola], "last name, first name, id");
    assert!(!ids.contains(&stranger));

    let zola_row = &row.learners()[2];
    assert_eq!(zola_row.status(), "in_progress");
    assert_eq!(zola_row.progress(), 50);
    assert!(zola_row.started_at().is_some());
    assert!(zola_row.last_activity_at().is_some());
    assert!(zola_row.completed_at().is_none());
    let anne = &row.learners()[0];
    assert_eq!((anne.first_name(), anne.last_name()), ("Anne", "Blum"));
    assert_eq!(anne.status(), "not_started");
    assert_eq!(anne.progress(), 0);
    assert!(anne.last_reminder_at().is_some());
    assert!(row.learners()[1].last_reminder_at().is_none());

    let page: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(course as u64, 2, 2))
        .await
        .unwrap();
    assert_eq!(page.total(), 3);
    assert_eq!(page.learners().len(), 1);
    assert_eq!(page.learners()[0].user_id(), zola);

    let past: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(course as u64, 2, 100))
        .await
        .unwrap();
    assert_eq!(past.total(), 3);
    assert!(past.learners().is_empty());
}

#[tokio::test]
async fn test_learners_of_an_unknown_or_empty_formation() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let unknown: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(
            u64::from(u32::MAX),
            50,
            0,
        ))
        .await
        .unwrap();
    assert!(!unknown.course_exists());
    assert_eq!(unknown.total(), 0);

    let course = create_course(&fx, "Sans apprenant", "Sans apprenant").await;
    let empty: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(course as u64, 50, 0))
        .await
        .unwrap();
    assert!(empty.course_exists());
    assert_eq!(empty.total(), 0);
    assert!(empty.learners().is_empty());
}

#[tokio::test]
async fn test_a_completed_formation_counts_as_full_progress() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Un module", "Un module").await;
    let only = create_module(&fx, course, "Seul module", "Contenu", 1).await;
    let agent = create_named_user(&fx, "Jeanne", "Martin").await;
    enrol(&fx, agent, course).await;
    complete(&fx, agent, course, only).await;

    let row: FormationLearnersRow = db
        .fetch_one(&GetFormationLearnersQueryView::new(course as u64, 50, 0))
        .await
        .unwrap();
    let learner = &row.learners()[0];
    assert_eq!(learner.status(), "completed");
    assert_eq!(learner.progress(), 100);
    assert!(learner.completed_at().is_some());
}
