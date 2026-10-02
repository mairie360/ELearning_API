//! MAIR-423: `/ready` answers `200` only when Postgres and Redis answer.

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::{web, App};
use elearning_api::endpoints::ready::{check_postgres, check_redis, ready, ReadinessError};
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

/// An `AppState` whose Redis listens nowhere (port 1).
async fn state_without_redis(pg_url: &str) -> AppState {
    AppState::with_keycloak("redis://127.0.0.1:1".to_string(), pg_url.to_string(), None).await
}

#[actix_web::test]
async fn postgres_check_passes_on_a_live_database() {
    let (_container, pg_url) = get_shared_db().await;
    let state = state_without_redis(pg_url).await;

    assert_eq!(check_postgres(&state).await, Ok(()));
    assert_eq!(check_redis(&state).await, Err(ReadinessError::Redis));
}

#[actix_web::test]
async fn ready_is_503_while_a_dependency_is_down() {
    let (_container, pg_url) = get_shared_db().await;
    for (pg_url, expected) in [
        (
            "postgres://user:password@127.0.0.1:1/db".to_string(),
            "PostgreSQL is unreachable.",
        ),
        (pg_url.to_string(), "Redis is unreachable."),
    ] {
        let state = state_without_redis(&pg_url).await;
        let app =
            test::init_service(App::new().app_data(web::Data::new(state)).service(ready)).await;

        let response =
            test::call_service(&app, TestRequest::get().uri("/ready").to_request()).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(test::read_body(response).await, expected);
    }
}
