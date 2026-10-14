//! Logging the cause of an error before answering a generic one (MAIR-395).

/// Builds a `map_err` closure that logs `err` at the `error` level, with the
/// handler name `context` as the `context` field, then yields `mapped`.
///
/// The endpoint error enums deliberately answer a generic body (a `500` only
/// says "An error occurred while accessing the database."), so without this
/// log the cause of a production error would be lost. Only the error is
/// logged, never the values of the request (MAIR-290).
pub fn log_error<E: std::fmt::Display, T>(context: &'static str, mapped: T) -> impl FnOnce(E) -> T {
    move |err| {
        tracing::error!(context, "{err}");
        mapped
    }
}
