//! Per-user rate limiting of `/api` (MAIR-425).
//!
//! The API is only reached through the BFFs, so limiting by peer IP would
//! throttle a whole BFF pod: the key is the authenticated user instead, read
//! from the `AuthenticatedUser` that `JwtMiddleware` inserted. The limiter must
//! therefore run **inside** `JwtMiddleware` (wrapped before it on the scope).

use actix_governor::governor::clock::{Clock, DefaultClock, QuantaInstant};
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::governor::NotUntil;
use actix_governor::{
    GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::dev::ServiceRequest;
use actix_web::http::header::{ContentType, RETRY_AFTER};
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, HttpResponse, HttpResponseBuilder};
use mairie360_api_lib::env_manager::get_env_var;
use mairie360_api_lib::security::AuthenticatedUser;

/// Requests per second a user regains, when `RATE_LIMIT_PER_SECOND` is unset.
pub const DEFAULT_PER_SECOND: u64 = 50;
/// Requests a user may send at once, when `RATE_LIMIT_BURST` is unset.
pub const DEFAULT_BURST: u32 = 100;

/// Keys the limiter on the id of the authenticated user.
#[derive(Debug, Clone, Copy)]
pub struct UserKeyExtractor;

impl KeyExtractor for UserKeyExtractor {
    type Key = u64;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<AuthenticatedUser>()
            .map(|user| user.id)
            // Only reachable if the limiter is mounted outside `JwtMiddleware`.
            .ok_or_else(|| {
                SimpleKeyExtractionError::new("Unauthorized: No JWT token provided.")
                    .set_status_code(StatusCode::UNAUTHORIZED)
            })
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait = negative
            .wait_time_from(DefaultClock::default().now())
            .as_secs()
            .max(1);
        // actix-governor already set `Retry-After` / `X-RateLimit-After`, but in whole seconds
        // rounded down: at the default 50 requests per second the wait is 20 ms and the header
        // said `0`, telling the client to retry at once. Both now carry the same wait as the
        // body, at least 1 s (MAIR-474).
        response
            .insert_header((RETRY_AFTER, wait))
            .insert_header(("x-ratelimit-after", wait))
            .content_type(ContentType::plaintext())
            .body(format!("Too many requests, retry in {wait} s."))
    }
}

/// Limiter configuration from `RATE_LIMIT_PER_SECOND` (requests regained per
/// second, default [`DEFAULT_PER_SECOND`]) and `RATE_LIMIT_BURST` (bucket
/// size, default [`DEFAULT_BURST`]). Build it **once**, outside the
/// `HttpServer::new` closure, so every worker shares the same buckets.
///
/// # Panics
///
/// Panics when a variable is set but is not a positive integer.
pub fn config_from_env() -> GovernorConfig<UserKeyExtractor, NoOpMiddleware> {
    let per_second = positive_env("RATE_LIMIT_PER_SECOND").unwrap_or(DEFAULT_PER_SECOND);
    let burst = positive_env("RATE_LIMIT_BURST")
        .map(|burst| u32::try_from(burst).unwrap_or(u32::MAX))
        .unwrap_or(DEFAULT_BURST);
    config(per_second, burst)
}

/// Limiter configuration: `burst` requests at once, then `per_second`.
///
/// # Panics
///
/// Panics when `per_second` or `burst` is `0`.
pub fn config(per_second: u64, burst: u32) -> GovernorConfig<UserKeyExtractor, NoOpMiddleware> {
    assert!(
        per_second > 0 && burst > 0,
        "the rate limit and the burst must be positive"
    );
    GovernorConfigBuilder::default()
        .requests_per_second(per_second)
        .burst_size(burst)
        .key_extractor(UserKeyExtractor)
        .finish()
        .expect("the rate limit and the burst must be positive")
}

fn positive_env(name: &str) -> Option<u64> {
    get_env_var(name).map(|value| match value.trim().parse::<u64>() {
        Ok(number) if number > 0 => number,
        _ => panic!("{name} must be a positive integer, got {value:?}"),
    })
}
