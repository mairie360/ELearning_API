//! Server-side logging, through the `log` facade.
//!
//! [`init`] installs `env_logger` with one JSON object per line on stderr
//! (`ts`, `level`, `target`, `message`), so a log collector can filter on the
//! level and the handler without parsing free text. The filter comes from
//! `RUST_LOG` (default `info`: startup, one line per request from `request_log::request_logger`
//! `Logger`, and every error below).

use std::io::Write;

/// Installs the JSON logger. Call once, first thing in `main`.
pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let line = serde_json::json!({
                "ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "level": record.level().as_str(),
                "target": record.target(),
                "message": record.args().to_string(),
            });
            writeln!(buf, "{line}")
        })
        .init();
}

/// Builds a `map_err` closure that logs `err` at the `error` level, with the
/// handler name `context` as the log target, then yields `mapped`.
///
/// The endpoint error enums deliberately answer a generic body (a `500` only
/// says "An error occurred while accessing the database."), so without this
/// log the cause of a production error would be lost.
pub fn log_error<E: std::fmt::Display, T>(context: &'static str, mapped: T) -> impl FnOnce(E) -> T {
    move |err| {
        log::error!(target: context, "{err}");
        mapped
    }
}
