use elearning_api::logging::log_error;

#[test]
fn log_error_yields_the_mapped_value() {
    let result: Result<(), &str> = Err("connection refused");
    assert_eq!(result.map_err(log_error("test_context", 500)), Err(500));
}

mod json_lines {
    use std::io::Write;
    use std::sync::{Arc, Mutex, Once};

    use elearning_api::logging::{layer, log_error};
    use serde_json::Value;
    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::layer::SubscriberExt;

    /// Collects what the log layer writes.
    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl Write for Captured {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'writer> MakeWriter<'writer> for Captured {
        type Writer = Self;

        fn make_writer(&'writer self) -> Self::Writer {
            self.clone()
        }
    }

    /// Runs `log` under the log layer and returns the JSON lines it wrote.
    fn lines(log: impl FnOnce()) -> Vec<Value> {
        static LOG_BRIDGE: Once = Once::new();
        LOG_BRIDGE.call_once(|| {
            tracing_log::LogTracer::init().expect("no other logger in this test binary");
        });
        let captured = Captured::default();
        let subscriber = tracing_subscriber::registry().with(layer(captured.clone()));
        tracing::subscriber::with_default(subscriber, log);
        let output = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
        output
            .lines()
            .map(|line| serde_json::from_str(line).expect("one JSON object per line"))
            .collect()
    }

    #[test]
    fn a_log_record_keeps_the_format_and_the_target_of_log_error() {
        let lines = lines(|| {
            let result: Result<(), &str> = Err("connection refused");
            let _ = result.map_err(log_error("trigger_get_formations", 500));
        });
        let [line] = lines.as_slice() else {
            panic!("one line expected: {lines:?}");
        };
        let keys: Vec<&str> = line
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["level", "message", "target", "ts"]);
        assert_eq!(line["level"], "ERROR");
        assert_eq!(line["target"], "trigger_get_formations");
        assert_eq!(line["message"], "connection refused");
    }

    #[test]
    fn a_tracing_event_gets_the_same_format_and_its_fields() {
        let lines = lines(|| tracing::warn!(attempt = 2, "PostgreSQL unreachable"));
        let [line] = lines.as_slice() else {
            panic!("one line expected: {lines:?}");
        };
        assert_eq!(line["level"], "WARN");
        assert_eq!(line["message"], "PostgreSQL unreachable");
        assert_eq!(line["attempt"], "2");
        assert!(line["ts"].as_str().is_some_and(|ts| ts.ends_with('Z')));
    }

    #[test]
    fn below_the_default_level_nothing_is_written() {
        assert!(lines(|| log::debug!("hidden")).is_empty());
    }
}
