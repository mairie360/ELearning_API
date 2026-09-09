use crate::common::get_smart_db;
use crate::queries::fixtures::create_course;
use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;
use elearning_api::database::formations::get_my_formations::view::{
    FormationSummaryRow, GetMyFormationsQueryView,
};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_register_user_to_formation_success_and_idempotent() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;

    let view = RegisterUserToFormationQueryView::new(alice_id as u64, course_id as u64);
    db.execute(view).await.expect("first registration failed");

    // Registering the same pair twice must be a no-op, not an error.
    let view = RegisterUserToFormationQueryView::new(alice_id as u64, course_id as u64);
    db.execute(view).await.expect("second registration failed");

    let list_view = GetMyFormationsQueryView::new(alice_id as u64);
    let rows = db
        .fetch_all::<FormationSummaryRow, _>(&list_view)
        .await
        .expect("query failed");
    assert_eq!(
        rows.iter().filter(|row| row.id() == course_id).count(),
        1,
        "registering twice must not duplicate the row"
    );
}
