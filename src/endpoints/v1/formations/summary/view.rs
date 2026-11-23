use utoipa::ToSchema;

use crate::endpoints::v1::formations::get::view::Status;

/// Training time of the caller on one formation during the year.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct SummaryFormation {
    /// Id of the formation.
    #[schema(example = 4)]
    pub id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    pub name: String,
    /// Current progress of the caller (whatever the year).
    pub status: Status,
    /// Seconds spent on the formation during the year: the durations of the contents of the
    /// modules completed that year.
    #[schema(example = 1800)]
    pub time_spent_seconds: u64,
    /// Completion of the formation, `null` while it is not completed.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-03-09T14:05:00Z")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Training time of the caller over a calendar year.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct SummaryResultView {
    /// The year of the figures.
    #[schema(example = 2026)]
    pub year: i32,
    /// Yearly training goal of an agent, in minutes, set for the whole instance.
    #[schema(example = 600)]
    pub annual_goal_minutes: u32,
    /// Seconds of training of the year, every formation together.
    #[schema(example = 5400)]
    pub time_spent_seconds: u64,
    /// Number of formations completed during the year.
    #[schema(example = 2)]
    pub completed_count: u64,
    /// Every formation the caller is enrolled in, by increasing id.
    pub formations: Vec<SummaryFormation>,
}
