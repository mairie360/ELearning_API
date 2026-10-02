use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// `SELECT 1`, the round trip of the readiness probe (`GET /ready`) and of the
/// startup check in `main.rs`. Never cached: it must reach Postgres every time.
/// Read it with `fetch_scalar::<i32, _>`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PingQueryView {
    params: Vec<QueryParam>,
}

impl PingQueryView {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ApiRequestDto for PingQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT 1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
