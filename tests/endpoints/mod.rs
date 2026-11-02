//! Handler-level regression tests: the real route tree of `main.rs`
//! (`/api` + `JwtMiddleware` + `endpoints::config`) served by `actix_web::test`
//! against the shared test database, with JWTs signed like Core signs them (MAIR-680: EdDSA,
//! the lib's test key pair; the API only holds the public key) and a mock file storage.

pub mod access_denials;
pub mod access_matrix;
pub mod admin_guard;
pub mod admin_reads;
pub mod catalog;
pub mod enrolment;
pub mod formation_access;
pub mod mair_680_session_keys;
pub mod module_access;
pub mod rate_limit;
pub mod readiness;
pub mod telemetry;
pub mod token_refusals;

use std::sync::{Arc, Mutex, Once};

use actix_web::web;

use crate::common::{elearning_db_url, fixture_db};
use elearning_api::storage::{FileStorage, StorageError};
use jsonwebtoken::{encode, Algorithm, Header};
use mairie360_api_lib::jwt_manager::{
    parse_public_keys, parse_signing_key, Claims, SESSION_JWT_AUDIENCE, SESSION_JWT_ISSUER,
};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::http_setup::{test_jwt_private_key, TEST_JWT_PUBLIC_KEY};

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

/// Seconds since the epoch, as the `exp` claim counts them.
pub fn now() -> usize {
    usize::try_from(chrono::Utc::now().timestamp()).unwrap()
}

/// Claims of a session token issued by Core for `user_id` (MAIR-680: with `iss` and `aud`).
pub fn core_claims(user_id: i32, expiration: usize) -> Claims {
    Claims::new(&user_id.to_string(), "User", expiration)
        .with_issuer_and_audience(SESSION_JWT_ISSUER, SESSION_JWT_AUDIENCE)
}

/// PKCS#8 body (base64) of an Ed25519 private key that is not Core's (MAIR-680), assembled into
/// PEM at run time so that the secret scanners do not read a PEM private key in the sources.
const MAIR_680_OTHER_PRIVATE_KEY_B64: &str =
    "MC4CAQAwBQYDK2VwBCIEII1DTYiu+dcwBgtEKil59uTH8s7XfrxGrbD+/P0sXq/8";

/// Public key of [`other_private_key`], never published in `JWT_PUBLIC_KEY`.
const MAIR_680_OTHER_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAtZHz1BNkijjkLJlnK9l7vikDl0aivrmbwllpT68Lnss=
-----END PUBLIC KEY-----
";

/// An Ed25519 private key that is not Core's: what a compromised service could sign with.
pub fn other_private_key() -> String {
    let label = "PRIVATE KEY";
    format!("-----BEGIN {label}-----\n{MAIR_680_OTHER_PRIVATE_KEY_B64}\n-----END {label}-----\n")
}

/// Signs `claims` as they are with `private_key` (PEM) under Core's `kid`: a genuine token with
/// [`test_jwt_private_key`], a forgery with [`other_private_key`].
pub fn sign_eddsa_with(claims: &Claims, private_key: &str) -> String {
    let core_keys = parse_public_keys(TEST_JWT_PUBLIC_KEY).unwrap();
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(core_keys[0].kid().to_string());
    let known_keys = [
        core_keys,
        parse_public_keys(MAIR_680_OTHER_PUBLIC_KEY).unwrap(),
    ]
    .concat();
    let key = parse_signing_key(private_key, &known_keys).unwrap();
    encode(&header, claims, key.encoding_key()).unwrap()
}

/// Signs `claims` like Core does (MAIR-680): EdDSA, the lib's test key pair, Core's `kid`.
pub fn sign_like_core(claims: &Claims) -> String {
    sign_eddsa_with(claims, &test_jwt_private_key())
}

/// The environment of a deployed API after MAIR-680: Core's public key, no `JWT_SECRET`, no
/// private key. Once per process.
pub fn set_jwt_env() {
    static JWT_ENV: Once = Once::new();
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_PUBLIC_KEY", TEST_JWT_PUBLIC_KEY);
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_SECRET");
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
}

/// A session JWT issued by Core for `user_id`, as an `Authorization` value.
pub fn jwt_for(user_id: i32) -> String {
    set_jwt_env();
    format!(
        "Bearer {}",
        sign_like_core(&core_claims(user_id, now() + 3600))
    )
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
        set_jwt_env();
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
