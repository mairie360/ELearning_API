use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, get_shared_db_as};

/// Postgres role the API connects with in every deployment (Database `api_roles.sql`, MAIR-114).
pub const ELEARNING_DB_ROLE: &str = "elearning_api";

/// The Redis cache is not needed by the tests: none of the query views declares a `cache_key`,
/// so `SmartDatabase` never reaches Redis. It still gets an instance (lazy pool, failures ignored).
const TEST_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// URL of the shared test database as [`ELEARNING_DB_ROLE`] (MAIR-556): the application code
/// under test (`AppState`, query views) goes through it, so a query the role is not granted fails
/// here with `permission denied` instead of in staging.
pub async fn elearning_db_url() -> String {
    get_shared_db_as(ELEARNING_DB_ROLE).await
}

async fn smart_db(url: &str) -> SmartDatabase {
    let db = Database::new(url).await;
    let redis = Redis::new(TEST_REDIS_URL);
    SmartDatabase::new(db, redis)
}

/// [`SmartDatabase`] on the shared test database, connected as [`ELEARNING_DB_ROLE`]: run the
/// query views under test through it.
pub async fn get_smart_db() -> SmartDatabase {
    smart_db(&elearning_db_url().await).await
}

/// [`SmartDatabase`] on the shared test database with the superuser URL of `get_shared_db()`,
/// for the fixtures only (seeding `users`, which the API role may not insert into). Never run the
/// code under test through it.
pub async fn fixture_db() -> SmartDatabase {
    let (_container, superuser_url) = get_shared_db().await;
    smart_db(superuser_url).await
}
