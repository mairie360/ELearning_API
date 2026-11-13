//! Input validation shared by every request body of the API (MAIR-243: same extractor as
//! Core, Project and Message).
//!
//! A request view implements [`Validate`] and the handler extracts it with [`ValidatedJson`]
//! instead of `web::Json`: an invalid value is rejected with a `400 Bad Request` (plain-text body
//! naming the field) before the handler runs, so it never reaches Postgres.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use serde::de::DeserializeOwned;

/// User ids are `INTEGER` in Postgres: anything above would fail at the database.
pub const MAX_USER_ID: u64 = i32::MAX as u64;

/// Why a request value was rejected; its text is the body of the `400` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(String);

impl ValidationError {
    pub fn new(field: &str, reason: &str) -> Self {
        Self(format!("Invalid `{field}`: {reason}"))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Implemented by every request view extracted with [`ValidatedJson`].
pub trait Validate {
    /// # Errors
    ///
    /// Returns the first field that does not satisfy its constraints.
    fn validate(&self) -> Result<(), ValidationError>;
}

/// A user id: between 1 and [`MAX_USER_ID`].
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when the id is out of range.
pub fn check_user_id(field: &str, id: u64) -> Result<(), ValidationError> {
    if id == 0 || id > MAX_USER_ID {
        return Err(ValidationError::new(
            field,
            &format!("must be between 1 and {MAX_USER_ID}"),
        ));
    }
    Ok(())
}

fn bad_request(error: &ValidationError) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(error.to_string())
}

/// `web::Json<T>` followed by [`Validate::validate`]: answers `400` when either fails.
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json = web::Json::<T>::from_request(req, payload);
        Box::pin(async move {
            let value = json.await?.into_inner();
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_is_bounded_to_the_integer_column() {
        assert!(check_user_id("user_id", 1).is_ok());
        assert!(check_user_id("user_id", MAX_USER_ID).is_ok());
        assert!(check_user_id("user_id", 0).is_err());
        assert!(check_user_id("user_id", MAX_USER_ID + 1).is_err());
    }

    #[test]
    fn the_error_names_the_field() {
        let error = check_user_id("user_id", 0).unwrap_err();
        assert!(error.to_string().starts_with("Invalid `user_id`"));
    }
}
