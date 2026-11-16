//! Input validation shared by every request body of the API (MAIR-243: same extractor as
//! Core, Project and Message).
//!
//! A request view implements [`Validate`] and the handler extracts it with [`ValidatedJson`]
//! instead of `web::Json`: an invalid value, a value of the wrong type or an unknown field is
//! rejected with a `400 Bad Request` (plain-text body naming the field) before the handler runs, so
//! it never reaches Postgres.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use actix_web::{dev::Payload, error::JsonPayloadError, web, FromRequest, HttpRequest};
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

/// Text of a body that does not deserialize: the field and what was expected, **never the value
/// received** nor the name of an unknown field (MAIR-290: serde quotes both, and the answer ends up
/// in logs and spans). `path` is the field serde stopped at (`.` for the body itself).
fn describe_body_error(error: &serde_path_to_error::Error<serde_json::Error>) -> String {
    let path = error.path().to_string();
    let inner = error.inner().to_string();
    let reason = if inner.starts_with("missing field ") {
        // The name is the server's own: the field the body lacks.
        inner
    } else if let Some((_, expected)) = inner.rsplit_once(", expected ") {
        // "invalid type: string \"x\", expected u64" / "unknown field `x`, expected `user_id`".
        if inner.starts_with("unknown field") {
            format!("unknown field, expected {expected}")
        } else {
            format!("expected {expected}")
        }
    } else {
        "invalid value".to_owned()
    };
    // The path of an unknown field is the key the client sent: not repeated either.
    if path == "." || reason.starts_with("unknown field") {
        format!("Invalid request body: {reason}")
    } else {
        format!("Invalid `{path}`: {reason}")
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

/// `web::Json<T>` followed by [`Validate::validate`]: answers `400` when either fails. The JSON is
/// read by `web::Json` (content type, size limit, syntax) and then deserialized into `T` with the
/// path of the field kept, so a wrong type or an unknown field names the field in the answer.
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
        let json = web::Json::<serde_json::Value>::from_request(req, payload);
        Box::pin(async move {
            let raw = json.await?.into_inner();
            // `JsonPayloadError::Deserialize`, like `web::Json`: the spans describe it by its type.
            let value: T = serde_path_to_error::deserialize(raw).map_err(|e| {
                JsonPayloadError::Deserialize(serde::de::Error::custom(describe_body_error(&e)))
            })?;
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Probe {
        user_id: u64,
    }

    impl Validate for Probe {
        fn validate(&self) -> Result<(), ValidationError> {
            check_user_id("user_id", self.user_id)
        }
    }

    /// The text of the `400` that `ValidatedJson<Probe>` answers to `body`, `None` when it accepts it.
    async fn rejection_of(body: &str) -> Option<String> {
        let (req, mut payload) = TestRequest::post()
            .insert_header(("content-type", "application/json"))
            .set_payload(body.to_owned())
            .to_http_parts();
        match ValidatedJson::<Probe>::from_request(&req, &mut payload).await {
            Ok(_) => None,
            Err(error) => {
                assert_eq!(error.error_response().status(), 400, "{body}");
                Some(error.to_string())
            }
        }
    }

    /// MAIR-243: a body that does not deserialize names the field (it used to be a position), and
    /// MAIR-290: without quoting what the client sent.
    #[actix_web::test]
    async fn a_wrong_type_unknown_field_or_missing_field_is_named() {
        assert_eq!(rejection_of(r#"{"user_id": 42}"#).await, None);
        for (body, field) in [
            (r#"{"user_id": "abc"}"#, "user_id"),
            (r#"{"user_id": null}"#, "user_id"),
            (r#"{"user_id": -1}"#, "user_id"),
            (
                r#"{"user_id": 42, "unexpected": true}"#,
                "unknown field, expected `user_id`",
            ),
            (r#"{}"#, "missing field `user_id`"),
            (r#"{"user": 1}"#, "unknown field, expected `user_id`"),
        ] {
            let text = rejection_of(body)
                .await
                .unwrap_or_else(|| panic!("{body} accepted"));
            assert!(text.contains(field), "{body}: {text}");
        }
        // Never the value nor the name of an unknown field (MAIR-290).
        for body in [
            r#"{"user_id": "gdpr.marker@example.com"}"#,
            r#"{"gdpr.marker@example.com": 1}"#,
        ] {
            let text = rejection_of(body).await.unwrap();
            assert!(!text.contains("gdpr.marker"), "{body}: {text}");
        }
        assert!(rejection_of(r#"{"user_id": 0}"#)
            .await
            .unwrap()
            .contains("Invalid `user_id`"));
        assert!(rejection_of("not json").await.is_some());
    }

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
