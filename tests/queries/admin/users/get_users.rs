use crate::common::get_smart_db;
use crate::queries::fixtures::create_user;
use elearning_api::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID, BOB_ID};

#[tokio::test]
async fn test_get_users_lists_active_users() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");
    let bob_id = *BOB_ID.get().expect("Bob ID missing");

    let view = GetUsersQueryView::new(200, 0);
    let rows = db
        .fetch_all::<UserRow, _>(&view)
        .await
        .expect("query failed");

    assert!(rows.iter().any(|row| row.id() == alice_id));
    // Bob is archived by `setup_archived_user_test`, so he must be excluded.
    assert!(!rows.iter().any(|row| row.id() == bob_id));
}

#[tokio::test]
async fn test_get_users_pages_in_id_order() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    // At least three active users, whatever the other tests left.
    for _ in 0..3 {
        create_user(&db).await;
    }

    let first_page = db
        .fetch_all::<UserRow, _>(&GetUsersQueryView::new(2, 0))
        .await
        .expect("query failed");
    let second_page = db
        .fetch_all::<UserRow, _>(&GetUsersQueryView::new(2, 2))
        .await
        .expect("query failed");

    assert_eq!(first_page.len(), 2);
    assert!(!second_page.is_empty());
    assert!(first_page[0].id() < first_page[1].id());
    assert!(first_page[1].id() < second_page[0].id());

    let past_the_end = db
        .fetch_all::<UserRow, _>(&GetUsersQueryView::new(2, 1_000_000))
        .await
        .expect("query failed");
    assert!(past_the_end.is_empty());
}
