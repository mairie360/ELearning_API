//! Handler-level regression tests: the real route tree of `main.rs`
//! (`/api` + `JwtMiddleware` + `endpoints::config`) served by `actix_web::test`
//! against the shared test database, with forged JWTs and a mock file storage.

pub mod admin_guard;
pub mod formation_access;
pub mod module_access;

use std::sync::{Arc, Mutex, Once};

use actix_web::web;
use elearning_api::storage::{FileStorage, StorageError};
use mairie360_api_lib::jwt_manager::generate_jwt;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

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

/// The pieces a handler test needs: the app state (whose `SmartDatabase` also
/// seeds fixtures) and the mock storage handed to the app.
pub struct TestContext {
    pub state: web::Data<AppState>,
    pub storage: Arc<MockFileStorage>,
}

impl TestContext {
    pub async fn new() -> Self {
        let (_container, pg_url) = get_shared_db().await;
        // Redis is not needed: none of the query views declares a cache key.
        let state = AppState::new("redis://127.0.0.1:6379".to_string(), pg_url.to_string()).await;
        Self {
            state: web::Data::new(state),
            storage: Arc::new(MockFileStorage::default()),
        }
    }

    pub fn db(&self) -> &SmartDatabase {
        self.state.get_smart_db()
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
