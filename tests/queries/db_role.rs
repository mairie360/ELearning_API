//! The code under test connects as `elearning_api`, never as the superuser (MAIR-556): a
//! superuser ignores the grants of Database's `api_grants.sql`, so a query the API may not run in
//! staging would pass here.

use crate::common::get_smart_db;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;

/// A raw statement without parameters, run through the same `SmartDatabase` as the query views.
#[derive(serde::Deserialize)]
struct RawQuery {
    #[serde(skip)]
    sql: &'static str,
}

impl ApiRequestDto for RawQuery {
    fn query_sql(&self) -> &'static str {
        self.sql
    }

    fn query_params(&self) -> &[QueryParam] {
        &[]
    }
}

/// The connected role, read through `to_jsonb` like every query view.
#[derive(serde::Deserialize, serde::Serialize)]
struct Role {
    name: String,
    superuser: bool,
}

/// Runs `sql`, whose rows are discarded: only the outcome matters.
async fn run(db: &SmartDatabase, sql: &'static str) -> Result<(), ApiLibError> {
    db.fetch_all::<serde_json::Value, _>(&RawQuery { sql })
        .await
        .map(drop)
}

/// SQLSTATE of a refused statement (`42501` = insufficient_privilege).
fn sqlstate(error: &ApiLibError) -> Option<String> {
    match error {
        ApiLibError::Database(DbError::Sqlx(error)) => error
            .as_database_error()
            .and_then(|error| error.code())
            .map(|code| code.into_owned()),
        _ => None,
    }
}

#[tokio::test]
async fn the_code_under_test_runs_as_elearning_api() {
    let db = get_smart_db().await;

    let role: Role = db
        .fetch_one(&RawQuery {
            sql:
                "SELECT to_jsonb(t) FROM (SELECT current_user::TEXT AS name, rolsuper AS superuser \
                  FROM pg_roles WHERE rolname = current_user) t",
        })
        .await
        .unwrap();

    assert_eq!(role.name, "elearning_api");
    assert!(!role.superuser);
}

#[tokio::test]
async fn elearning_api_is_held_to_its_grants() {
    let db = get_smart_db().await;

    // The API never reads the sessions of the core domain: the grants refuse it, as in staging.
    let error = run(
        &db,
        "SELECT to_jsonb(t) FROM (SELECT id FROM sessions LIMIT 1) t",
    )
    .await
    .expect_err("elearning_api must not read sessions");
    assert_eq!(sqlstate(&error).as_deref(), Some("42501"));

    // Nor the columns of `users` outside its column grants.
    let error = run(
        &db,
        "SELECT to_jsonb(t) FROM (SELECT password FROM users LIMIT 1) t",
    )
    .await
    .expect_err("elearning_api must not read users.password");
    assert_eq!(sqlstate(&error).as_deref(), Some("42501"));
}
