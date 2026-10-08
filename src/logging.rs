//! Server-side logging, through the `log` facade.
//!
//! [`layer`] writes one JSON object per line on stderr
//! (`ts`, `level`, `target`, `message`), so a log collector can filter on the
//! level and the handler without parsing free text. The filter comes from
//! `RUST_LOG` (default `info`: startup, one line per request from actix's
//! `Logger`, and every error below).

use std::fmt;

use serde_json::{Map, Value};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, MakeWriter};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// The log layer: one JSON object per line on `writer` (stderr in `main`), filtered by `RUST_LOG`
/// (default `info`). `telemetry::init` installs it, with the `log` records bridged to `tracing`
/// and the trace export next to it (MAIR-503).
#[must_use]
pub fn layer<S, W>(writer: W) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt::layer()
        .event_format(JsonLine)
        .with_writer(writer)
        .with_filter(filter)
}

/// Formats an event as `{"level", "message", "target", "ts"}`, plus its other fields. The spans
/// around the event are not printed: the request span holds the query string and the client
/// address (MAIR-290).
struct JsonLine;

impl<S, N> FormatEvent<S, N> for JsonLine
where
    S: Subscriber + for<'span> LookupSpan<'span>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // A `log` record keeps its own target (`log_error` sets the trigger fn) once normalized.
        let normalized = event.normalized_metadata();
        let metadata = normalized.as_ref().unwrap_or_else(|| event.metadata());
        let mut line = Fields::default();
        event.record(&mut line);
        let mut line = line.0;
        line.insert(
            "ts".to_string(),
            Value::from(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        );
        line.insert("level".to_string(), Value::from(metadata.level().as_str()));
        line.insert("target".to_string(), Value::from(metadata.target()));
        line.entry("message").or_insert_with(|| Value::from(""));
        writeln!(writer, "{}", Value::Object(line))
    }
}

/// The fields of an event, minus the `log.*` ones `tracing-log` adds to bridged records.
#[derive(Default)]
struct Fields(Map<String, Value>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if !field.name().starts_with("log.") {
            self.0.insert(field.name().to_string(), Value::from(value));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if !field.name().starts_with("log.") {
            self.0
                .insert(field.name().to_string(), Value::from(format!("{value:?}")));
        }
    }
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
