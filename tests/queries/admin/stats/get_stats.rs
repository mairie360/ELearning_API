//! The indicators read every row: they run on a database of their own (MAIR-1002).

use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::isolated::isolated_db;
use mairie360_api_lib::test_setup::redis_setup::isolated_redis;

use crate::common::ELEARNING_DB_ROLE;
use crate::queries::fixtures::{
    archive_user, complete, create_course, create_module, create_user, enrol, set_course_meta,
};
use elearning_api::database::admin::formations::delete_course::view::DeleteCourseQueryView;
use elearning_api::database::admin::stats::get_stats::view::{GetStatsQueryView, StatsRow};
use elearning_api::endpoints::v1::admin::stats::get::endpoint::load_stats;

async fn smart_db(url: &str) -> SmartDatabase {
    SmartDatabase::new(
        Database::new(url).await,
        Redis::new("redis://127.0.0.1:6379"),
    )
}

#[tokio::test]
async fn test_get_stats_counts_the_whole_instance() {
    let isolated = isolated_db().await;
    let fx = smart_db(&isolated.url()).await;
    let db = smart_db(&isolated.url_as(ELEARNING_DB_ROLE)).await;
    let before: StatsRow = db.fetch_one(&GetStatsQueryView::new()).await.unwrap();

    let mandatory = create_course(&fx, "Obligatoire", "Obligatoire").await;
    set_course_meta(&fx, mandatory, true, "2001-01-01", "", "internal").await;
    let first = create_module(&fx, mandatory, "Module 1", "Contenu", 1).await;
    create_module(&fx, mandatory, "Module 2", "Contenu", 2).await;
    let platform = create_course(&fx, "Plateforme", "Plateforme").await;
    set_course_meta(&fx, platform, false, "", "", "platform").await;
    let platform_module = create_module(&fx, platform, "Module", "Contenu", 1).await;
    let (late, started, done, archived, _idle) = (
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
        create_user(&fx).await,
    );
    for user in [late, started, archived] {
        enrol(&fx, user, mandatory).await;
    }
    enrol(&fx, done, platform).await;
    complete(&fx, started, mandatory, first).await;
    complete(&fx, done, platform, platform_module).await;
    archive_user(&fx, archived).await;

    let row: StatsRow = db.fetch_one(&GetStatsQueryView::new()).await.unwrap();
    assert_eq!(row.internal_courses(), before.internal_courses() + 1);
    assert_eq!(row.platform_courses(), before.platform_courses() + 1);
    assert_eq!(
        row.mandatory_internal_courses(),
        before.mandatory_internal_courses() + 1
    );
    // Five new accounts, the archived one is not an agent; `_idle` is enrolled nowhere.
    assert_eq!(row.agents(), before.agents() + 4);
    assert_eq!(row.participants(), before.participants() + 3);
    assert_eq!(row.late_mandatory(), before.late_mandatory() + 2);

    let mandatory_stats = row
        .by_formation()
        .iter()
        .find(|f| f.id() == mandatory)
        .unwrap();
    assert!(mandatory_stats.is_mandatory());
    assert_eq!(mandatory_stats.source(), "internal");
    assert_eq!(mandatory_stats.enrolled(), 2);
    assert_eq!(mandatory_stats.completed(), 0);
    assert!((mandatory_stats.average_progress() - 25.0).abs() < 1e-9);
    let platform_stats = row
        .by_formation()
        .iter()
        .find(|f| f.id() == platform)
        .unwrap();
    assert_eq!(platform_stats.completed(), 1);
    assert!((platform_stats.average_progress() - 100.0).abs() < 1e-9);

    let late_agents: Vec<_> = row
        .late_agents()
        .iter()
        .filter(|a| a.formation_id() == mandatory)
        .collect();
    assert_eq!(late_agents.len(), 2);
    assert_eq!(late_agents[0].user_id(), late.min(started));
    assert_eq!(late_agents[0].formation_name(), "Obligatoire");
    assert_eq!(late_agents[0].deadline().to_string(), "2001-01-01");

    let without_service = row
        .by_service()
        .iter()
        .find(|s| s.service_id().is_none())
        .expect("the agents without a service");
    let before_without = before
        .by_service()
        .iter()
        .find(|s| s.service_id().is_none())
        .map_or(0, |s| s.agents());
    assert_eq!(without_service.agents(), before_without + 4);
}

fn formation_ids(row: &StatsRow) -> Vec<i32> {
    row.by_formation().iter().map(|f| f.id()).collect()
}

/// The promote-staging k6 gate crossed its thresholds: computed on every call, the indicators
/// saturated the database under load. They are now cached in Redis, shared by every admin, until
/// an admin write on the formations or the end of the TTL.
#[tokio::test]
async fn mair_1203_the_indicators_are_cached_until_an_admin_write() {
    let isolated = isolated_db().await;
    let fx = smart_db(&isolated.url()).await;
    let db = SmartDatabase::new(
        Database::new(&isolated.url_as(ELEARNING_DB_ROLE)).await,
        isolated_redis().await,
    );
    let listed = create_course(&fx, "Lue", "Lue").await;

    let first = load_stats(&db).await.unwrap();
    assert!(formation_ids(&first).contains(&listed));

    // A formation written behind the API's back is not seen: the page comes from the cache.
    let unseen = create_course(&fx, "Hors API", "Hors API").await;
    let cached = load_stats(&db).await.unwrap();
    assert_eq!(formation_ids(&cached), formation_ids(&first));

    // An admin write of the API invalidates it once committed.
    let deleted: bool = db
        .fetch_scalar(&DeleteCourseQueryView::new(u64::try_from(listed).unwrap()))
        .await
        .unwrap();
    assert!(deleted);
    let fresh = load_stats(&db).await.unwrap();
    assert!(!formation_ids(&fresh).contains(&listed));
    assert!(formation_ids(&fresh).contains(&unseen));
}

/// Concurrent misses of one process share one computation (the expiry of the cached entry must not
/// run the aggregate once per waiting request): they all answer the same row.
#[tokio::test]
async fn mair_1203_concurrent_misses_share_one_computation() {
    let isolated = isolated_db().await;
    let db = SmartDatabase::new(
        Database::new(&isolated.url_as(ELEARNING_DB_ROLE)).await,
        isolated_redis().await,
    );
    let rows = futures_util::future::join_all((0..8).map(|_| load_stats(&db))).await;
    let first = formation_ids(rows[0].as_ref().unwrap());
    for row in &rows {
        assert_eq!(formation_ids(row.as_ref().unwrap()), first);
    }
}
