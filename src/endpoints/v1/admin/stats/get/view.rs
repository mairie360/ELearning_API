use utoipa::ToSchema;

use crate::endpoints::v1::formations::catalog::view::CourseSource;

/// Participation of the agents of one service of the mairie.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ServiceStats {
    /// Id of the service, `null` for the agents without a service.
    #[schema(example = 3)]
    pub service_id: Option<u64>,
    /// Name of the service, `null` for the agents without a service.
    #[schema(example = "Civil status")]
    pub service_name: Option<String>,
    /// Non-archived agents of the service.
    #[schema(example = 12)]
    pub agents: u64,
    /// Those enrolled in at least one formation.
    #[schema(example = 9)]
    pub participants: u64,
    /// `participants / agents` in percent, one decimal (0 without agent).
    #[schema(example = 75.0)]
    pub participation_rate: f64,
}

/// Enrolments of one formation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct FormationStats {
    /// Id of the formation.
    #[schema(example = 4)]
    pub id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    pub name: String,
    /// Origin of the formation.
    pub source: CourseSource,
    /// `true` for a mandatory formation.
    #[schema(example = true)]
    pub is_mandatory: bool,
    /// Non-archived agents enrolled.
    #[schema(example = 40)]
    pub enrolled: u64,
    /// Those who completed it.
    #[schema(example = 25)]
    pub completed: u64,
    /// Average progress of the enrolled agents, in percent, one decimal (0 without enrolment).
    #[schema(example = 71.3)]
    pub average_progress: f64,
}

/// An agent late on a mandatory formation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LateAgent {
    /// Core API id of the agent.
    #[schema(example = 42)]
    pub user_id: u64,
    /// First name of the agent.
    #[schema(example = "Jean")]
    pub first_name: String,
    /// Last name of the agent.
    #[schema(example = "Martin")]
    pub last_name: String,
    /// Id of the formation.
    #[schema(example = 4)]
    pub formation_id: u64,
    /// Title of the formation.
    #[schema(example = "GDPR for local government staff")]
    pub formation_name: String,
    /// Deadline of the formation (`YYYY-MM-DD`), past.
    #[schema(value_type = String, format = Date, example = "2026-06-30")]
    pub deadline: chrono::NaiveDate,
    /// Progress of the agent, in percent, one decimal.
    #[schema(example = 50.0)]
    pub progress: f64,
    /// Latest reminder sent to the agent about this formation, `null` without one.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub last_reminder_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Indicators of the e-learning administration.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetStatsResultView {
    /// Formations made by the mairie (`source = internal`).
    #[schema(example = 18)]
    pub internal_courses: u64,
    /// Formations provided with Mairie 360 (`source = platform`).
    #[schema(example = 6)]
    pub platform_courses: u64,
    /// Mandatory formations among the internal ones.
    #[schema(example = 4)]
    pub mandatory_internal_courses: u64,
    /// Non-archived accounts.
    #[schema(example = 120)]
    pub agents: u64,
    /// Non-archived accounts enrolled in at least one formation.
    #[schema(example = 90)]
    pub participants: u64,
    /// `participants / agents` in percent, one decimal (0 without agent).
    #[schema(example = 75.0)]
    pub participation_rate: f64,
    /// Average progress over every enrolment, in percent, one decimal: completed modules / modules
    /// of the formation (100 once completed).
    #[schema(example = 62.4)]
    pub average_completion: f64,
    /// Enrolments on a mandatory formation whose deadline is past, not completed.
    #[schema(example = 7)]
    pub late_mandatory: u64,
    /// Participation per service of the mairie, by service name (the agents without a service
    /// last).
    pub by_service: Vec<ServiceStats>,
    /// Every formation, by increasing id.
    pub by_formation: Vec<FormationStats>,
    /// The 500 most overdue late enrolments (`late_mandatory` counts them all), by deadline, then
    /// formation, then agent.
    pub late_agents: Vec<LateAgent>,
}
