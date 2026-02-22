use gunbc_ir::transport::github::IssueLifecycleStage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClaimLedger {
    pub claims: BTreeMap<String, ClaimRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimRecord {
    pub owner: String,
    pub claimed_at_epoch_ms: u128,
    pub lease_expires_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimAcquireResult {
    Acquired,
    AlreadyOwned,
    StaleReclaimed { previous_owner: String },
    Conflict { current_owner: String },
}

pub fn claim_slot_key(issue_id: u64, stage: IssueLifecycleStage) -> String {
    format!("issue:{issue_id}:stage:{}", stage.as_label())
}

pub fn try_acquire_claim(
    ledger: &mut ClaimLedger,
    slot_key: &str,
    owner: &str,
    now_epoch_ms: u128,
    lease_ttl_ms: u128,
) -> ClaimAcquireResult {
    match ledger.claims.get(slot_key) {
        None => {
            ledger.claims.insert(
                slot_key.to_string(),
                ClaimRecord {
                    owner: owner.to_string(),
                    claimed_at_epoch_ms: now_epoch_ms,
                    lease_expires_at_epoch_ms: now_epoch_ms.saturating_add(lease_ttl_ms),
                },
            );
            ClaimAcquireResult::Acquired
        }
        Some(existing) if existing.owner == owner => ClaimAcquireResult::AlreadyOwned,
        Some(existing) if existing.lease_expires_at_epoch_ms <= now_epoch_ms => {
            let previous_owner = existing.owner.clone();
            ledger.claims.insert(
                slot_key.to_string(),
                ClaimRecord {
                    owner: owner.to_string(),
                    claimed_at_epoch_ms: now_epoch_ms,
                    lease_expires_at_epoch_ms: now_epoch_ms.saturating_add(lease_ttl_ms),
                },
            );
            ClaimAcquireResult::StaleReclaimed { previous_owner }
        }
        Some(existing) => ClaimAcquireResult::Conflict {
            current_owner: existing.owner.clone(),
        },
    }
}

pub fn release_claim(ledger: &mut ClaimLedger, slot_key: &str, owner: &str) -> bool {
    match ledger.claims.get(slot_key) {
        Some(record) if record.owner == owner => {
            ledger.claims.remove(slot_key);
            true
        }
        _ => false,
    }
}

pub fn heartbeat_claim(
    ledger: &mut ClaimLedger,
    slot_key: &str,
    owner: &str,
    now_epoch_ms: u128,
    lease_ttl_ms: u128,
) -> bool {
    match ledger.claims.get_mut(slot_key) {
        Some(record) if record.owner == owner => {
            record.lease_expires_at_epoch_ms = now_epoch_ms.saturating_add(lease_ttl_ms);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_claim_conflicts_when_active_owner_exists() {
        let mut ledger = ClaimLedger::default();
        let key = claim_slot_key(7, IssueLifecycleStage::Design);
        let first = try_acquire_claim(&mut ledger, &key, "worker-a", 1000, 500);
        assert_eq!(first, ClaimAcquireResult::Acquired);

        let second = try_acquire_claim(&mut ledger, &key, "worker-b", 1200, 500);
        assert_eq!(
            second,
            ClaimAcquireResult::Conflict {
                current_owner: "worker-a".to_string()
            }
        );
    }

    #[test]
    fn acquire_claim_reclaims_stale_slot() {
        let mut ledger = ClaimLedger::default();
        let key = claim_slot_key(7, IssueLifecycleStage::Design);
        let _ = try_acquire_claim(&mut ledger, &key, "worker-a", 1000, 100);
        let reclaimed = try_acquire_claim(&mut ledger, &key, "worker-b", 1201, 100);
        assert_eq!(
            reclaimed,
            ClaimAcquireResult::StaleReclaimed {
                previous_owner: "worker-a".to_string()
            }
        );
    }

    #[test]
    fn release_claim_is_owner_scoped() {
        let mut ledger = ClaimLedger::default();
        let key = claim_slot_key(1, IssueLifecycleStage::Idea);
        let _ = try_acquire_claim(&mut ledger, &key, "worker-a", 1000, 1000);
        assert!(!release_claim(&mut ledger, &key, "worker-b"));
        assert!(release_claim(&mut ledger, &key, "worker-a"));
        assert!(!ledger.claims.contains_key(&key));
    }

    #[test]
    fn heartbeat_claim_extends_owner_lease() {
        let mut ledger = ClaimLedger::default();
        let key = claim_slot_key(2, IssueLifecycleStage::Implementing);
        let _ = try_acquire_claim(&mut ledger, &key, "worker-a", 1000, 100);
        assert!(heartbeat_claim(&mut ledger, &key, "worker-a", 1080, 250));
        assert_eq!(
            ledger
                .claims
                .get(&key)
                .map(|record| record.lease_expires_at_epoch_ms),
            Some(1330)
        );
        assert!(!heartbeat_claim(&mut ledger, &key, "worker-b", 1100, 250));
    }
}
