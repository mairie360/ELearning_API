//! `/api` is rate limited per authenticated user.

use std::num::NonZeroU32;

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use mairie360_api_lib::security::{rate_limit_middleware, RateLimitConfig, RateLimiter};
use serial_test::serial;

/// The lib's defaults (`RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST` unset).
const DEFAULT_PER_SECOND: u64 = 20;
const DEFAULT_BURST: u32 = 50;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::create_user;

#[actix_web::test]
async fn each_user_has_its_own_bucket() {
    let ctx = TestContext::new().await;
    // A burst of 2, then one request per second: the third request in a row is refused.
    let limiter = RateLimiter::new(RateLimitConfig {
        per_second: NonZeroU32::MIN,
        burst: NonZeroU32::new(2).unwrap(),
        trust_forwarded: false,
    });
    let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = ctx.storage.clone();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(ctx.state.clone())
            .app_data(limiter)
            .service(
                actix_web::web::scope("/api")
                    .app_data(actix_web::web::Data::from(storage))
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(elearning_api::endpoints::config),
            ),
    )
    .await;
    let greedy = jwt_for(create_user(ctx.db()).await);
    let other = jwt_for(create_user(ctx.db()).await);
    let get = |jwt: &str| {
        TestRequest::get()
            .uri("/api/v1/formations/")
            .insert_header(("Authorization", jwt.to_string()))
            .to_request()
    };

    for _ in 0..2 {
        assert_eq!(
            test::call_service(&app, get(&greedy)).await.status(),
            StatusCode::OK
        );
    }
    let refused = test::call_service(&app, get(&greedy)).await;
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(refused.headers().contains_key("retry-after"));
    assert!(test::read_body(refused)
        .await
        .starts_with(b"Too many requests, retry in "));

    // Another user is not affected.
    assert_eq!(
        test::call_service(&app, get(&other)).await.status(),
        StatusCode::OK
    );
}

/// The production budget: `RateLimiter::from_env()` without overrides, mounted like `main.rs`.
/// A user gets the whole burst of `DEFAULT_BURST`, then `429` with a numeric `Retry-After`, while
/// another user is still served.
#[actix_web::test]
#[serial(rate_limit_env)]
async fn the_production_budget_refuses_a_user_past_its_burst() {
    std::env::remove_var("RATE_LIMIT_PER_SECOND");
    std::env::remove_var("RATE_LIMIT_BURST");
    std::env::remove_var("RATE_LIMIT_TRUST_FORWARDED");
    let ctx = TestContext::new().await;
    let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = ctx.storage.clone();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(ctx.state.clone())
            .app_data(RateLimiter::from_env())
            .service(
                actix_web::web::scope("/api")
                    .app_data(actix_web::web::Data::from(storage))
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(elearning_api::endpoints::config),
            ),
    )
    .await;
    let greedy = jwt_for(create_user(ctx.db()).await);
    let other = jwt_for(create_user(ctx.db()).await);
    let get = |jwt: &str| {
        TestRequest::get()
            .uri("/api/v1/formations/")
            .insert_header(("Authorization", jwt.to_string()))
            .to_request()
    };

    let started = std::time::Instant::now();
    let mut served: u64 = 0;
    let refused = loop {
        let response = test::call_service(&app, get(&greedy)).await;
        if response.status() != StatusCode::OK {
            break response;
        }
        served += 1;
        assert!(served < 10_000, "the limiter never refused");
    };
    // The bucket refills while the burst is sent: at most one request per elapsed 1/DEFAULT_PER_SECOND.
    let refilled =
        u64::try_from(started.elapsed().as_millis()).unwrap() * DEFAULT_PER_SECOND / 1000 + 1;
    assert!(
        served >= u64::from(DEFAULT_BURST) && served <= u64::from(DEFAULT_BURST) + refilled,
        "{served} requests served before the 429"
    );
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u64 = refused
        .headers()
        .get("retry-after")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(retry_after >= 1, "Retry-After: {retry_after}");
    assert_eq!(
        refused.headers().get("x-ratelimit-after"),
        refused.headers().get("retry-after")
    );

    assert_eq!(
        test::call_service(&app, get(&other)).await.status(),
        StatusCode::OK
    );
}

/// A disabled limiter (`RATE_LIMIT_PER_SECOND=0`) lets every request through.
#[actix_web::test]
async fn mair_688_a_disabled_limiter_lets_every_request_through() {
    let ctx = TestContext::new().await;
    let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = ctx.storage.clone();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(ctx.state.clone())
            .app_data(RateLimiter::disabled())
            .service(
                actix_web::web::scope("/api")
                    .app_data(actix_web::web::Data::from(storage))
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(elearning_api::endpoints::config),
            ),
    )
    .await;
    let user = jwt_for(create_user(ctx.db()).await);
    for _ in 0..10 {
        let request = TestRequest::get()
            .uri("/api/v1/formations/")
            .insert_header(("Authorization", user.clone()))
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            StatusCode::OK
        );
    }
}
