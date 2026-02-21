use super::retry::{retry_ready, RetryState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileEntry {
    pub intake_key: String,
    pub claim_slot: String,
    pub claim_owner: Option<String>,
    pub claim_expires_at_epoch_ms: Option<u128>,
    pub awaiting_approval: bool,
    pub retry: RetryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconcileAction {
    ReleaseClaim {
        intake_key: String,
        claim_slot: String,
        owner: String,
        reason: String,
    },
    Terminalize {
        intake_key: String,
        reason: String,
    },
    ReadyToRun {
        intake_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcilePlan {
    pub actions: Vec<ReconcileAction>,
}

pub fn reconcile_entries(entries: &[ReconcileEntry], now_epoch_ms: u128) -> ReconcilePlan {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| left.intake_key.cmp(&right.intake_key));

    let mut actions = Vec::new();
    for entry in sorted {
        if entry.awaiting_approval {
            if let Some(owner) = entry.claim_owner.clone() {
                actions.push(ReconcileAction::ReleaseClaim {
                    intake_key: entry.intake_key.clone(),
                    claim_slot: entry.claim_slot.clone(),
                    owner,
                    reason: "awaiting_approval".to_string(),
                });
            }
            continue;
        }

        if entry.retry.attempts >= entry.retry.budget {
            actions.push(ReconcileAction::Terminalize {
                intake_key: entry.intake_key.clone(),
                reason: "retry_budget_exhausted".to_string(),
            });
            continue;
        }

        let claim_is_stale = match entry.claim_expires_at_epoch_ms {
            Some(expiry) => expiry <= now_epoch_ms,
            None => false,
        };
        if claim_is_stale {
            if let Some(owner) = entry.claim_owner.clone() {
                actions.push(ReconcileAction::ReleaseClaim {
                    intake_key: entry.intake_key.clone(),
                    claim_slot: entry.claim_slot.clone(),
                    owner,
                    reason: "stale_claim".to_string(),
                });
            }
            continue;
        }

        if retry_ready(&entry.retry, now_epoch_ms) {
            actions.push(ReconcileAction::ReadyToRun {
                intake_key: entry.intake_key.clone(),
            });
        }
    }

    ReconcilePlan { actions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_releases_claim_when_awaiting_approval() {
        let plan = reconcile_entries(
            &[ReconcileEntry {
                intake_key: "a".to_string(),
                claim_slot: "issue:1:stage:design".to_string(),
                claim_owner: Some("worker-1".to_string()),
                claim_expires_at_epoch_ms: Some(2000),
                awaiting_approval: true,
                retry: RetryState::default(),
            }],
            1500,
        );
        assert_eq!(
            plan.actions,
            vec![ReconcileAction::ReleaseClaim {
                intake_key: "a".to_string(),
                claim_slot: "issue:1:stage:design".to_string(),
                owner: "worker-1".to_string(),
                reason: "awaiting_approval".to_string(),
            }]
        );
    }

    #[test]
    fn reconcile_terminalizes_when_retry_budget_exhausted() {
        let plan = reconcile_entries(
            &[ReconcileEntry {
                intake_key: "a".to_string(),
                claim_slot: "issue:1:stage:design".to_string(),
                claim_owner: None,
                claim_expires_at_epoch_ms: None,
                awaiting_approval: false,
                retry: RetryState {
                    attempts: 3,
                    budget: 3,
                    next_retry_at_epoch_ms: None,
                    last_error: Some("oops".to_string()),
                },
            }],
            1500,
        );
        assert_eq!(
            plan.actions,
            vec![ReconcileAction::Terminalize {
                intake_key: "a".to_string(),
                reason: "retry_budget_exhausted".to_string(),
            }]
        );
    }

    #[test]
    fn reconcile_is_deterministic_by_intake_key_order() {
        let entries = vec![
            ReconcileEntry {
                intake_key: "b".to_string(),
                claim_slot: "issue:2:stage:design".to_string(),
                claim_owner: None,
                claim_expires_at_epoch_ms: None,
                awaiting_approval: false,
                retry: RetryState::default(),
            },
            ReconcileEntry {
                intake_key: "a".to_string(),
                claim_slot: "issue:1:stage:design".to_string(),
                claim_owner: None,
                claim_expires_at_epoch_ms: None,
                awaiting_approval: false,
                retry: RetryState::default(),
            },
        ];
        let plan = reconcile_entries(&entries, 1000);
        assert_eq!(
            plan.actions,
            vec![
                ReconcileAction::ReadyToRun {
                    intake_key: "a".to_string()
                },
                ReconcileAction::ReadyToRun {
                    intake_key: "b".to_string()
                },
            ]
        );
    }
}
