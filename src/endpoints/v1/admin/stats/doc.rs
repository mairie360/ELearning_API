use crate::endpoints::v1::admin::stats::get::endpoint::__path_get_stats;
use crate::endpoints::v1::admin::stats::get::view::{
    FormationStats, GetStatsResultView, LateAgent, ServiceStats,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_stats),
    components(schemas(GetStatsResultView, ServiceStats, FormationStats, LateAgent))
)]
pub struct StatsDoc;
