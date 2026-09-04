use crate::common::get_smart_db;
use elearning_api::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID, BOB_ID};

#[tokio::test]
async fn test_get_users_lists_active_users() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");
    let bob_id = *BOB_ID.get().expect("Bob ID missing");

    let view = GetUsersQueryView::new();
    let rows = db
        .fetch_all::<UserRow, _>(&view)
        .await
        .expect("query failed");

    assert!(rows.iter().any(|row| row.id() == alice_id));
    // Bob is archived by `setup_archived_user_test`, so he must be excluded.
    assert!(!rows.iter().any(|row| row.id() == bob_id));
}
