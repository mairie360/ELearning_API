//! MAIR-425: `/api` is rate limited per authenticated user.

use actix_governor::Governor;
use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use elearning_api::rate_limit;

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

#[test]
#[should_panic(expected = "positive")]
fn a_zero_rate_is_refused() {
    let _ = rate_limit::config(0, 10);
}
