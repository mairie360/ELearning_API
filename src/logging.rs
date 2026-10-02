//! Server-side logging of the errors the handlers hide from the client.

/// Builds a `map_err` closure that logs `err` on stderr, tagged with
/// `context`, then yields `mapped`.
///
/// The endpoint error enums deliberately answer a generic body (a `500` only
/// says "An error occurred while accessing the database."), so without this
/// log the cause of a production error would be lost.
pub fn log_error<E: std::fmt::Display, T>(context: &'static str, mapped: T) -> impl FnOnce(E) -> T {
    move |err| {
        eprintln!("[{context}] {err}");
        mapped
    }
}
