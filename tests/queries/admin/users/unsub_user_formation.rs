use crate::common::get_smart_db;
use crate::queries::fixtures::create_course;
use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;
use elearning_api::database::admin::users::unsub_user_formation::view::UnsubUserFormationQueryView;
use elearning_api::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_unsub_user_formation_removes_registration() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let register_view = RegisterUserToFormationQueryView::new(alice_id as u64, course_id as u64);
    db.execute(register_view)
        .await
        .expect("registration failed");

    let unsub_view = UnsubUserFormationQueryView::new(alice_id as u64, course_id as u64);
    db.execute(unsub_view).await.expect("unsub failed");

    let list_view = GetMyFormationsQueryView::new(alice_id as u64);
    let rows = db
        .fetch_all::<FormationSummaryRow, _>(&list_view)
        .await
        .expect("query failed");
    assert!(!rows.iter().any(|row| row.id() == course_id));
}

#[tokio::test]
async fn test_unsub_user_formation_is_a_noop_when_not_registered() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    let unsub_view = UnsubUserFormationQueryView::new(alice_id as u64, course_id as u64);
    let result = db.execute(unsub_view).await;

    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}
