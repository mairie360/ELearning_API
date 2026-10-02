use elearning_api::logging::log_error;

#[test]
fn log_error_yields_the_mapped_value() {
    let result: Result<(), &str> = Err("connection refused");
    assert_eq!(result.map_err(log_error("test_context", 500)), Err(500));
}
