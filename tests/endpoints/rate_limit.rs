//! MAIR-425: `/api` is rate limited per authenticated user.

use actix_governor::Governor;
use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use elearning_api::rate_limit;

use serial_test::serial;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::create_user;

#[actix_web::test]
async fn each_user_has_its_own_bucket() {
    let ctx = TestContext::new().await;
    // A burst of 2, then one request every 10 s (slower than
    // `rate_limit::config` allows, so the test cannot race the refill): the
    // third request in a row is refused.
    let limiter = actix_governor::GovernorConfigBuilder::default()
        .seconds_per_request(10)
        .burst_size(2)
        .key_extractor(rate_limit::UserKeyExtractor)
        .finish()
        .unwrap();
    let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = ctx.storage.clone();
    let app = test::init_service(
        actix_web::App::new().app_data(ctx.state.clone()).service(
            actix_web::web::scope("/api")
                .app_data(actix_web::web::Data::from(storage))
                .wrap(Governor::new(&limiter))
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

/// The production budget (MAIR-474): `config_from_env()` without overrides, mounted like `main.rs`.
/// A user gets the whole burst of `DEFAULT_BURST`, then `429` with a numeric `Retry-After`, while
/// another user is still served.
#[actix_web::test]
#[serial]
async fn the_production_budget_refuses_a_user_past_its_burst() {
    std::env::remove_var("RATE_LIMIT_PER_SECOND");
    std::env::remove_var("RATE_LIMIT_BURST");
    let ctx = TestContext::new().await;
    let limiter = rate_limit::config_from_env();
    let storage: std::sync::Arc<dyn elearning_api::storage::FileStorage> = ctx.storage.clone();
    let app = test::init_service(
        actix_web::App::new().app_data(ctx.state.clone()).service(
            actix_web::web::scope("/api")
                .app_data(actix_web::web::Data::from(storage))
                .wrap(Governor::new(&limiter))
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
    let refilled = u64::try_from(started.elapsed().as_millis()).unwrap()
        * rate_limit::DEFAULT_PER_SECOND
        / 1000
        + 1;
    assert!(
        served >= u64::from(rate_limit::DEFAULT_BURST)
            && served <= u64::from(rate_limit::DEFAULT_BURST) + refilled,
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

#[test]
#[serial]
#[should_panic(expected = "RATE_LIMIT_BURST must be a positive integer")]
fn a_malformed_burst_is_refused_at_startup() {
    std::env::set_var("RATE_LIMIT_BURST", "a lot");
    let outcome = std::panic::catch_unwind(rate_limit::config_from_env);
    std::env::remove_var("RATE_LIMIT_BURST");
    std::panic::resume_unwind(outcome.unwrap_err());
}

#[test]
#[should_panic(expected = "positive")]
fn a_zero_rate_is_refused() {
    let _ = rate_limit::config(0, 10);
}
