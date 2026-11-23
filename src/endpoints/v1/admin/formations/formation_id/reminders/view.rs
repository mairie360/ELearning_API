use crate::endpoints::validation::{check_user_id, Validate, ValidationError};

/// Most agents one reminder request names.
pub const MAX_REMINDED_USERS: usize = 1000;

/// Agents to remind of a mandatory formation.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SendRemindersView {
    /// Core API ids of the agents to remind (at most 1000). Absent: every late agent of the
    /// formation (deadline past, not completed). Only enrolled agents who have not completed the
    /// formation are reminded; the other ids are ignored.
    #[schema(example = json!([42]))]
    #[serde(default)]
    user_ids: Option<Vec<u64>>,
}

impl SendRemindersView {
    pub fn new(user_ids: Option<Vec<u64>>) -> Self {
        Self { user_ids }
    }

    pub fn user_ids(&self) -> Option<&[u64]> {
        self.user_ids.as_deref()
    }
}

impl Validate for SendRemindersView {
    fn validate(&self) -> Result<(), ValidationError> {
        let Some(ids) = &self.user_ids else {
            return Ok(());
        };
        if ids.len() > MAX_REMINDED_USERS {
            return Err(ValidationError::new(
                "user_ids",
                &format!("at most {MAX_REMINDED_USERS} agents"),
            ));
        }
        for (index, id) in ids.iter().enumerate() {
            check_user_id(&format!("user_ids[{index}]"), *id)?;
        }
        Ok(())
    }
}

/// Number of reminders recorded.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SendRemindersResultView {
    /// One per agent reminded; 0 when nobody was to remind.
    #[schema(example = 3)]
    pub sent: u64,
}
