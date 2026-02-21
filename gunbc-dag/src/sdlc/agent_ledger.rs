use gunbc_ir::transport::agent::{AgentHandle, AgentStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AgentLedger {
    pub entries: BTreeMap<String, AgentLedgerRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLedgerRecord {
    pub intake_key: String,
    pub handle: AgentHandle,
    pub status: AgentStatus,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
    pub updated_at_epoch_ms: u128,
}

pub fn upsert_agent_record(
    ledger: &mut AgentLedger,
    intake_key: &str,
    handle: AgentHandle,
    status: AgentStatus,
    now_epoch_ms: u128,
) -> AgentUpsertOutcome {
    let key = intake_key.to_string();
    if let Some(existing) = ledger.entries.get(&key) {
        if existing.handle == handle && existing.status == status {
            return AgentUpsertOutcome::Noop;
        }
    }
    let is_update = ledger.entries.contains_key(&key);
    ledger.entries.insert(
        key,
        AgentLedgerRecord {
            intake_key: intake_key.to_string(),
            handle,
            status,
            pr_number: None,
            pr_url: None,
            updated_at_epoch_ms: now_epoch_ms,
        },
    );
    if is_update {
        AgentUpsertOutcome::Updated
    } else {
        AgentUpsertOutcome::Inserted
    }
}

pub fn update_agent_status(
    ledger: &mut AgentLedger,
    intake_key: &str,
    status: AgentStatus,
    now_epoch_ms: u128,
) -> Result<(), String> {
    let record = ledger
        .entries
        .get_mut(intake_key)
        .ok_or_else(|| format!("no agent record for intake_key '{intake_key}'"))?;
    record.status = status;
    record.updated_at_epoch_ms = now_epoch_ms;
    Ok(())
}

pub fn update_agent_pr(
    ledger: &mut AgentLedger,
    intake_key: &str,
    pr_number: u64,
    pr_url: &str,
    now_epoch_ms: u128,
) -> Result<(), String> {
    let record = ledger
        .entries
        .get_mut(intake_key)
        .ok_or_else(|| format!("no agent record for intake_key '{intake_key}'"))?;
    record.pr_number = Some(pr_number);
    record.pr_url = Some(pr_url.to_string());
    record.updated_at_epoch_ms = now_epoch_ms;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentUpsertOutcome {
    Inserted,
    Updated,
    Noop,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handle(key: &str) -> AgentHandle {
        AgentHandle {
            provider: "stub".into(),
            session_id: format!("stub-{key}"),
            intake_key: key.into(),
            spawned_at_epoch_ms: 1000,
        }
    }

    #[test]
    fn upsert_inserts_new_record() {
        let mut ledger = AgentLedger::default();
        let outcome = upsert_agent_record(
            &mut ledger,
            "key-1",
            test_handle("key-1"),
            AgentStatus::Running { progress: None },
            1000,
        );
        assert_eq!(outcome, AgentUpsertOutcome::Inserted);
        assert!(ledger.entries.contains_key("key-1"));
    }

    #[test]
    fn upsert_is_idempotent_for_same_state() {
        let mut ledger = AgentLedger::default();
        let handle = test_handle("key-1");
        let status = AgentStatus::Running { progress: None };
        upsert_agent_record(&mut ledger, "key-1", handle.clone(), status.clone(), 1000);
        let outcome = upsert_agent_record(&mut ledger, "key-1", handle, status, 1100);
        assert_eq!(outcome, AgentUpsertOutcome::Noop);
    }

    #[test]
    fn update_status_changes_existing_record() {
        let mut ledger = AgentLedger::default();
        upsert_agent_record(
            &mut ledger,
            "key-1",
            test_handle("key-1"),
            AgentStatus::Running { progress: None },
            1000,
        );
        update_agent_status(
            &mut ledger,
            "key-1",
            AgentStatus::Completed {
                branch: "feature/test".into(),
                commit_sha: "abc".into(),
            },
            2000,
        )
        .expect("update should succeed");
        let record = ledger.entries.get("key-1").unwrap();
        assert!(matches!(&record.status, AgentStatus::Completed { .. }));
    }

    #[test]
    fn update_status_fails_for_missing_key() {
        let mut ledger = AgentLedger::default();
        let err = update_agent_status(
            &mut ledger,
            "missing",
            AgentStatus::Running { progress: None },
            1000,
        )
        .unwrap_err();
        assert!(err.contains("no agent record"));
    }

    #[test]
    fn update_pr_records_number_and_url() {
        let mut ledger = AgentLedger::default();
        upsert_agent_record(
            &mut ledger,
            "key-1",
            test_handle("key-1"),
            AgentStatus::Completed {
                branch: "b".into(),
                commit_sha: "c".into(),
            },
            1000,
        );
        update_agent_pr(
            &mut ledger,
            "key-1",
            99,
            "https://github.com/test/repo/pull/99",
            2000,
        )
        .expect("update PR should succeed");
        let record = ledger.entries.get("key-1").unwrap();
        assert_eq!(record.pr_number, Some(99));
    }
}
