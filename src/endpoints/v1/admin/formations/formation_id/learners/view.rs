use crate::endpoints::v1::admin::users::ProgressStatus;

/// A non-archived agent enrolled in the formation, with their progress.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FormationLearner {
    /// Core API id of the agent.
    #[schema(example = 42)]
    pub user_id: u64,
    /// First name of the agent.
    #[schema(example = "Jean")]
    pub first_name: String,
    /// Last name of the agent.
    #[schema(example = "Dupont")]
    pub last_name: String,
    /// Progress of the agent on this formation, recomputed by the database.
    pub status: ProgressStatus,
    /// Completed modules over the modules of the formation, from 0 to 100, rounded down: 100
    /// once the formation is completed, 0 for a formation without module.
    #[schema(example = 50, minimum = 0, maximum = 100)]
    pub progress: u8,
    /// Date of the first completed module, or `null` if the agent completed none.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-02T08:30:00Z")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completion date of the formation, or `null` if it is not completed.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of the agent's last module write on this formation, or `null` if none.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-03T14:25:00Z")]
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of the last reminder an administrator recorded for the agent on this formation, or
    /// `null` if none.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub last_reminder_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// One page of the learners of a formation.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GetFormationLearnersResultView {
    /// Id of the formation.
    #[schema(example = 4)]
    pub formation_id: u64,
    /// Number of non-archived learners of the formation, all pages together.
    #[schema(example = 2)]
    pub total: u64,
    /// The page size used (`limit` once capped).
    #[schema(example = 50)]
    pub limit: u32,
    /// The `offset` asked for.
    #[schema(example = 0)]
    pub offset: u32,
    /// Learners of the page, by last name, first name and id. Empty past the last page.
    pub learners: Vec<FormationLearner>,
}
