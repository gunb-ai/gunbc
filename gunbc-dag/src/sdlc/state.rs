use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunStateLedger {
    pub entries: BTreeMap<String, RunStateRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunStateRecord {
    pub run_key: String,
    pub status: RunExecutionStatus,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunExecutionStatus {
    Completed,
    Failed,
}

pub fn should_replay_skip(
    ledger: &RunStateLedger,
    intake_key: &str,
    run_key: &str,
) -> bool {
    matches!(
        ledger.entries.get(intake_key),
        Some(RunStateRecord {
            run_key: existing_run_key,
            status: RunExecutionStatus::Completed,
            ..
        }) if existing_run_key == run_key
    )
}

pub fn mark_run_completed(
    ledger: &mut RunStateLedger,
    intake_key: &str,
    run_key: &str,
    now_epoch_ms: u128,
) {
    ledger.entries.insert(
        intake_key.to_string(),
        RunStateRecord {
            run_key: run_key.to_string(),
            status: RunExecutionStatus::Completed,
            updated_at_epoch_ms: now_epoch_ms,
        },
    );
}

pub fn mark_run_failed(
    ledger: &mut RunStateLedger,
    intake_key: &str,
    run_key: &str,
    now_epoch_ms: u128,
) {
    ledger.entries.insert(
        intake_key.to_string(),
        RunStateRecord {
            run_key: run_key.to_string(),
            status: RunExecutionStatus::Failed,
            updated_at_epoch_ms: now_epoch_ms,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_skip_only_when_same_run_key_completed() {
        let mut ledger = RunStateLedger::default();
        mark_run_completed(&mut ledger, "intent-a", "run-a", 1000);
        assert!(should_replay_skip(&ledger, "intent-a", "run-a"));
        assert!(!should_replay_skip(&ledger, "intent-a", "run-b"));
    }

    #[test]
    fn replay_skip_false_after_failed_state() {
        let mut ledger = RunStateLedger::default();
        mark_run_failed(&mut ledger, "intent-a", "run-a", 1000);
        assert!(!should_replay_skip(&ledger, "intent-a", "run-a"));
    }
}
