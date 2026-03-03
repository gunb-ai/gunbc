//! Execution ledger for runtime redundancy detection (C22-P2).
//!
//! While [`StaticFingerprint`] catches duplicate transport operations at
//! compile time, the execution ledger catches redundancy at test time —
//! when runtime values are available but static analysis couldn't prove
//! equivalence (e.g., `InputProvenance::Dynamic` keys).
//!
//! Usage: inject an `ExecutionLedger` into test harnesses, record each
//! transport execution, then assert no redundant operations at the end.

use gunbc_ir::OperationKey;
use std::collections::HashMap;

/// A single execution record in the ledger.
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    /// The service operation that was executed.
    pub operation: OperationKey,
    /// Hash of the resolved fingerprint (operation + concrete key values).
    pub fingerprint_hash: u64,
    /// The DAG node that performed this execution.
    pub node_id: String,
}

/// Violation detected when two records have matching operation + fingerprint
/// but originate from different nodes.
#[derive(Debug, Clone)]
pub struct RedundancyViolation {
    /// The first (earlier) execution record.
    pub first: ExecutionRecord,
    /// The second (later) execution record that duplicates the first.
    pub second: ExecutionRecord,
}

/// Tracks transport operations during execution for redundancy detection.
///
/// Each transport execute node records its resolved operation key and
/// fingerprint hash. After execution completes, `check_all()` scans for
/// pairs that performed identical work from different nodes.
#[derive(Debug, Default, Clone)]
pub struct ExecutionLedger {
    entries: Vec<ExecutionRecord>,
}

impl ExecutionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a transport execution. Returns a violation if this operation
    /// duplicates a prior entry with matching fingerprint.
    pub fn record(&mut self, record: ExecutionRecord) -> Option<RedundancyViolation> {
        for prior in &self.entries {
            if prior.operation == record.operation
                && prior.fingerprint_hash == record.fingerprint_hash
                && prior.node_id != record.node_id
            {
                return Some(RedundancyViolation {
                    first: prior.clone(),
                    second: record.clone(),
                });
            }
        }
        self.entries.push(record);
        None
    }

    /// Check all entries for redundancy violations.
    pub fn check_all(&self) -> Vec<RedundancyViolation> {
        let mut violations = Vec::new();
        let mut seen: HashMap<(OperationKey, u64), &ExecutionRecord> = HashMap::new();
        for entry in &self.entries {
            let key = (entry.operation.clone(), entry.fingerprint_hash);
            if let Some(prior) = seen.get(&key) {
                if prior.node_id != entry.node_id {
                    violations.push(RedundancyViolation {
                        first: (*prior).clone(),
                        second: entry.clone(),
                    });
                }
            } else {
                seen.insert(key, entry);
            }
        }
        violations
    }

    /// All recorded entries.
    pub fn entries(&self) -> &[ExecutionRecord] {
        &self.entries
    }

    /// Whether any executions have been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of recorded executions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(service: &str, op: &str, hash: u64, node: &str) -> ExecutionRecord {
        ExecutionRecord {
            operation: OperationKey::new(service, op),
            fingerprint_hash: hash,
            node_id: node.to_string(),
        }
    }

    #[test]
    fn no_violations_for_distinct_operations() {
        let mut ledger = ExecutionLedger::new();
        assert!(ledger
            .record(make_record("cargo", "Build", 1, "node_a"))
            .is_none());
        assert!(ledger
            .record(make_record("cargo", "Test", 1, "node_b"))
            .is_none());
        assert!(ledger.check_all().is_empty());
    }

    #[test]
    fn no_violation_for_same_operation_different_fingerprint() {
        let mut ledger = ExecutionLedger::new();
        assert!(ledger
            .record(make_record("cargo", "Build", 1, "node_a"))
            .is_none());
        assert!(ledger
            .record(make_record("cargo", "Build", 2, "node_b"))
            .is_none());
        assert!(ledger.check_all().is_empty());
    }

    #[test]
    fn detects_redundancy_on_record() {
        let mut ledger = ExecutionLedger::new();
        assert!(ledger
            .record(make_record("cargo", "Build", 42, "node_a"))
            .is_none());
        let violation = ledger
            .record(make_record("cargo", "Build", 42, "node_b"))
            .expect("should detect redundancy");
        assert_eq!(violation.first.node_id, "node_a");
        assert_eq!(violation.second.node_id, "node_b");
    }

    #[test]
    fn detects_redundancy_via_check_all() {
        let mut ledger = ExecutionLedger::new();
        // Bypass record() return to test check_all independently
        ledger.entries.push(make_record("fs", "Read", 99, "n1"));
        ledger.entries.push(make_record("fs", "Read", 99, "n2"));
        let violations = ledger.check_all();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].first.node_id, "n1");
        assert_eq!(violations[0].second.node_id, "n2");
    }

    #[test]
    fn same_node_same_fingerprint_is_not_a_violation() {
        let mut ledger = ExecutionLedger::new();
        assert!(ledger
            .record(make_record("cargo", "Build", 1, "node_a"))
            .is_none());
        // Same node executing again (e.g., loop iteration) is not redundancy
        assert!(ledger
            .record(make_record("cargo", "Build", 1, "node_a"))
            .is_none());
        assert!(ledger.check_all().is_empty());
    }

    #[test]
    fn len_and_is_empty() {
        let mut ledger = ExecutionLedger::new();
        assert!(ledger.is_empty());
        assert_eq!(ledger.len(), 0);
        ledger.record(make_record("a", "b", 0, "n"));
        assert!(!ledger.is_empty());
        assert_eq!(ledger.len(), 1);
    }
}
