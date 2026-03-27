use super::{ComplianceCheck, ComplianceStatus};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Compliance workflow step types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStep {
    KycInitiated,
    KycCompleted,
    AmlScreened,
    RiskAssessed,
    ManualReviewRequired,
    ManualReviewCompleted,
    Approved,
    Rejected,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub id: Uuid,
    pub check_id: Uuid,
    pub step: WorkflowStep,
    pub actor: String,
    pub notes: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

/// Determine the next workflow step based on a compliance check result.
pub fn next_step(check: &ComplianceCheck) -> WorkflowStep {
    match check.status {
        ComplianceStatus::Approved => WorkflowStep::Approved,
        ComplianceStatus::Blocked => WorkflowStep::Blocked,
        ComplianceStatus::Rejected => WorkflowStep::Rejected,
        ComplianceStatus::RequiresReview => WorkflowStep::ManualReviewRequired,
        ComplianceStatus::Pending => WorkflowStep::KycInitiated,
    }
}

/// Build a workflow event for a compliance check transition.
pub fn build_workflow_event(check: &ComplianceCheck, actor: &str) -> WorkflowEvent {
    WorkflowEvent {
        id: Uuid::new_v4(),
        check_id: check.id,
        step: next_step(check),
        actor: actor.to_string(),
        notes: check.notes.clone(),
        occurred_at: Utc::now(),
    }
}

/// Compliance workflow state machine — validates allowed transitions.
pub fn is_valid_transition(from: &ComplianceStatus, to: &ComplianceStatus) -> bool {
    use ComplianceStatus::*;
    matches!(
        (from, to),
        (Pending, Approved)
            | (Pending, Rejected)
            | (Pending, RequiresReview)
            | (Pending, Blocked)
            | (RequiresReview, Approved)
            | (RequiresReview, Rejected)
            | (RequiresReview, Blocked)
    )
}
