use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{
    archive_user, create_course, create_named_user, create_user, enrol,
};
use elearning_api::database::admin::users::get_users::view::{GetUsersQueryView, UserRow};
use mairie360_api_lib::test_setup::queries_setup::{ALICE_ID, BOB_ID};

#[tokio::test]
async fn test_get_users_lists_active_users() {
    let db = get_smart_db().await;
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
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    // At least three active users, whatever the other tests left.
    for _ in 0..3 {
        create_user(&fx).await;
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

/// A marker no other test writes in a name, so the search only finds this test's users.
fn marker() -> String {
    format!(
        "Mkr{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

#[tokio::test]
async fn test_mair_817_search_matches_names_whatever_the_case() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let tag = marker();
    let jean = create_named_user(&fx, "Jean", &format!("Dupont{tag}")).await;
    let amina = create_named_user(&fx, "Amina", &format!("Bensaid{tag}")).await;
    let archived = create_named_user(&fx, "Jean", &format!("Archive{tag}")).await;
    archive_user(&fx, archived).await;
    let search = |term: &str| GetUsersQueryView::filtered(200, 0, Some(term), None);

    let rows = db
        .fetch_all::<UserRow, _>(&search(&tag.to_lowercase()))
        .await
        .unwrap();
    let ids: Vec<i32> = rows.iter().map(UserRow::id).collect();
    assert_eq!(ids, vec![jean, amina], "id order, archived left out");
    assert_eq!(rows[0].first_name(), "Jean");
    assert_eq!(rows[0].last_name(), format!("Dupont{tag}"));
    assert_eq!(rows[0].name(), format!("Jean Dupont{tag}"));

    for term in [
        format!("JEAN DUPONT{tag}"),
        format!("dupont{} jean", tag.to_lowercase()),
    ] {
        let rows = db.fetch_all::<UserRow, _>(&search(&term)).await.unwrap();
        assert_eq!(
            rows.iter().map(UserRow::id).collect::<Vec<_>>(),
            vec![jean],
            "{term}"
        );
    }
    // `%` and `_` match themselves, not any character.
    for term in [format!("%{tag}"), format!("{tag}_")] {
        assert!(
            db.fetch_all::<UserRow, _>(&search(&term))
                .await
                .unwrap()
                .is_empty(),
            "{term}"
        );
    }
}

#[tokio::test]
async fn test_mair_817_exclude_formation_leaves_out_its_learners() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let tag = marker();
    let enrolled = create_named_user(&fx, "Inscrit", &tag).await;
    let free = create_named_user(&fx, "Libre", &tag).await;
    let course = create_course(&fx, "Exclusion", "Exclusion").await;
    enrol(&fx, enrolled, course).await;

    let rows = db
        .fetch_all::<UserRow, _>(&GetUsersQueryView::filtered(
            200,
            0,
            Some(&tag),
            Some(course as u64),
        ))
        .await
        .unwrap();
    assert_eq!(rows.iter().map(UserRow::id).collect::<Vec<_>>(), vec![free]);

    // An unknown formation leaves nobody out.
    let rows = db
        .fetch_all::<UserRow, _>(&GetUsersQueryView::filtered(
            200,
            0,
            Some(&tag),
            Some(u64::from(u32::MAX)),
        ))
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
}
