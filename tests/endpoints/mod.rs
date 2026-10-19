//! Handler-level regression tests: the real route tree of `main.rs`
//! (`/api` + `JwtMiddleware` + `endpoints::config`) served by `actix_web::test`
//! against the shared test database, with forged JWTs and a mock file storage.

pub mod access_denials;
pub mod admin_guard;
pub mod admin_reads;
pub mod catalog;
pub mod enrolment;
pub mod formation_access;
pub mod module_access;
pub mod rate_limit;
pub mod readiness;
pub mod telemetry;
pub mod token_refusals;

use std::sync::{Arc, Mutex, Once};

use actix_web::web;

use crate::common::{elearning_db_url, fixture_db};
use elearning_api::storage::{FileStorage, StorageError};
use mairie360_api_lib::jwt_manager::generate_jwt;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;

pub const PRESIGNED_URL: &str = "https://storage.test/presigned";

/// Records every key it is asked to sign, so a test can prove that a refused
/// request never produced a presigned URL.
#[derive(Default)]
pub struct MockFileStorage {
    pub calls: Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl FileStorage for MockFileStorage {
    async fn presigned_view_url(
        &self,
        key: &str,
        _content_type: &str,
    ) -> Result<String, StorageError> {
        self.calls.lock().unwrap().push(key.to_string());
        Ok(PRESIGNED_URL.to_string())
    }
}

/// Signs a JWT for `user_id` with the test secret.
pub fn jwt_for(user_id: i32) -> String {
    static JWT_ENV: Once = Once::new();
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_SECRET", "elearning-test-secret");
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
    let token = generate_jwt(&user_id.to_string(), "User").expect("failed to sign test JWT");
    format!("Bearer {token}")
}

/// The pieces a handler test needs: the app state, connected as the API's
/// Postgres role, the superuser database that seeds the fixtures, and the mock
/// storage handed to the app.
pub struct TestContext {
    pub state: web::Data<AppState>,
    pub storage: Arc<MockFileStorage>,
    fixtures: SmartDatabase,
}

impl TestContext {
    pub async fn new() -> Self {
        // The app runs as `elearning_api`, like in staging (MAIR-556). Redis is
        // not needed: none of the query views declares a cache key.
        let state = AppState::new(
            "redis://127.0.0.1:6379".to_string(),
            elearning_db_url().await,
        )
        .await;
        Self {
            state: web::Data::new(state),
            storage: Arc::new(MockFileStorage::default()),
            fixtures: fixture_db().await,
        }
    }

    /// Superuser database for the fixtures and the assertions on the seeded
    /// rows; the code under test goes through `state` instead.
    pub fn db(&self) -> &SmartDatabase {
        &self.fixtures
    }
}

/// Builds the app exactly like `main.rs` mounts `/api`.
#[macro_export]
macro_rules! init_app {
    ($ctx:expr) => {{
        let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = $ctx.storage.clone();
        actix_web::test::init_service(
            actix_web::App::new().app_data($ctx.state.clone()).service(
                actix_web::web::scope("/api")
                    .app_data(actix_web::web::Data::from(storage))
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(elearning_api::endpoints::config),
            ),
        )
        .await
    }};
}

/// Sends a request and returns its status, whether the handler answered or a
/// middleware (JWT, admin) rejected it with an error.
#[macro_export]
macro_rules! status_of {
    ($app:expr, $req:expr) => {{
        match actix_web::test::try_call_service(&$app, $req.to_request()).await {
            Ok(response) => response.status(),
            Err(error) => error.as_response_error().status_code(),
        }
    }};
}
