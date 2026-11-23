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

/// A label (name, title, category): not blank, at most `max` characters, no control character.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when the value breaks one of these rules.
pub fn check_label(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    check_length(field, value, max)?;
    if value.chars().any(char::is_control) {
        return Err(ValidationError::new(
            field,
            "must not contain control characters",
        ));
    }
    Ok(())
}

/// A free text (description): at most `max` characters, line breaks and tabs allowed, no other
/// control character.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when the value breaks one of these rules.
pub fn check_text(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    check_length(field, value, max)?;
    if value
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(ValidationError::new(
            field,
            "must not contain control characters other than line breaks and tabs",
        ));
    }
    Ok(())
}

fn check_length(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.chars().count() > max {
        return Err(ValidationError::new(
            field,
            &format!("must be at most {max} characters long"),
        ));
    }
    Ok(())
}

/// Longest URL accepted for a link content.
pub const MAX_URL_LENGTH: usize = 2048;

/// The URL of a link content (MAIR-703): an absolute `https://` URL with a host, without user
/// information, whitespace nor control character. `javascript:`, `data:`, `http:` and relative
/// URLs are refused, so the front can put it in an `href` as is.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when the value is not such a URL.
pub fn check_https_url(field: &str, value: &str) -> Result<(), ValidationError> {
    const SCHEME: &str = "https://";
    let refuse = || {
        Err(ValidationError::new(
            field,
            "must be an absolute https:// URL with a host",
        ))
    };
    check_length(field, value, MAX_URL_LENGTH)?;
    if value
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return refuse();
    }
    let Some(scheme) = value.get(..SCHEME.len()) else {
        return refuse();
    };
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return refuse();
    }
    let rest = &value[SCHEME.len()..];
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    // No `user@host`: `https://trusted.example@evil.example` reads as the first host.
    if authority.contains('@') {
        return refuse();
    }
    let host = match authority.strip_prefix('[') {
        // IPv6 literal: `[::1]:8443`.
        Some(v6) => v6.split(']').next().unwrap_or_default(),
        None => authority.split(':').next().unwrap_or_default(),
    };
    let valid_host = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '.' | '_' | ':'));
    if !valid_host {
        return refuse();
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

    #[test]
    fn mair_703_only_absolute_https_urls_are_accepted() {
        for url in [
            "https://www.example.org",
            "https://www.example.org/path?query=1#anchor",
            "HTTPS://example.org/",
            "https://example.org:8443/video",
            "https://[::1]:8443/",
        ] {
            assert!(check_https_url("url", url).is_ok(), "{url}");
        }
        for url in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "http://example.org",
            "//example.org/path",
            "/relative/path",
            "relative/path",
            "https://",
            "https:///path",
            "https://user@example.org",
            "https://exa mple.org",
            "https://example.org/a b",
            "https://example.org/\u{0}",
            "https://example.org\\@evil.org",
            "ftp://example.org",
            "",
        ] {
            let error = check_https_url("modules[0].files[1].url", url).unwrap_err();
            assert!(
                error
                    .to_string()
                    .starts_with("Invalid `modules[0].files[1].url`"),
                "{url}: {error}"
            );
        }
        let long = format!("https://example.org/{}", "a".repeat(MAX_URL_LENGTH));
        assert!(check_https_url("url", &long).is_err());
    }

    #[test]
    fn labels_and_texts_are_bounded_and_without_control_characters() {
        assert!(check_label("name", "Accueil du public", 255).is_ok());
        assert!(check_label("name", "  ", 255).is_err());
        assert!(check_label("name", "a\u{7}b", 255).is_err());
        assert!(check_label("name", &"é".repeat(255), 255).is_ok());
        assert!(check_label("name", &"é".repeat(256), 255).is_err());
        assert!(check_text("description", "", 10).is_ok());
        assert!(check_text("description", "line\nbreak\ttab", 20).is_ok());
        assert!(check_text("description", "nul\u{0}", 20).is_err());
        assert!(check_text("description", "too long", 3).is_err());
    }
}
