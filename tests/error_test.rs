//! `error::log_error` (MAIR-395): the cause of an error answered as a generic body is logged at the
//! `error` level with the handler name as context, then the mapped value is returned.

use std::fmt;
use std::sync::{Arc, Mutex};

use elearning_api::error::log_error;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Layer;

/// One captured event: its level and its fields, rendered as text.
#[derive(Debug)]
struct Captured {
    level: Level,
    fields: Vec<(String, String)>,
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Captured>>>);

struct Fields(Vec<(String, String)>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.push((field.name().to_string(), value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0
            .push((field.name().to_string(), format!("{value:?}")));
    }
}

impl<S: Subscriber> Layer<S> for Capture {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields(Vec::new());
        event.record(&mut fields);
        self.0.lock().unwrap().push(Captured {
            level: *event.metadata().level(),
            fields: fields.0,
        });
    }
}

fn captured(run: impl FnOnce()) -> Vec<Captured> {
    let capture = Capture::default();
    let subscriber = tracing_subscriber::registry().with(capture.clone());
    tracing::subscriber::with_default(subscriber, run);
    let events = std::mem::take(&mut *capture.0.lock().unwrap());
    events
}

fn field<'a>(event: &'a Captured, name: &str) -> Option<&'a str> {
    event
        .fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

#[test]
fn mair_395_log_error_yields_the_mapped_value() {
    let result: Result<(), &str> = Err("connection refused");
    assert_eq!(result.map_err(log_error("test_context", 500)), Err(500));
}

#[test]
fn mair_395_log_error_logs_the_cause_at_error_with_the_handler_as_context() {
    let events = captured(|| {
        let result: Result<(), &str> = Err("connection refused");
        let _ = result.map_err(log_error("trigger_get_formations", 500));
    });
    let [event] = events.as_slice() else {
        panic!("one event expected: {events:?}");
    };
    assert_eq!(event.level, Level::ERROR);
    assert_eq!(field(event, "context"), Some("trigger_get_formations"));
    assert_eq!(field(event, "message"), Some("connection refused"));
}

#[test]
fn mair_395_log_error_logs_nothing_on_success() {
    let events = captured(|| {
        let result: Result<(), &str> = Ok(());
        let _ = result.map_err(log_error("trigger_get_formations", 500));
    });
    assert!(events.is_empty(), "{events:?}");
}
