//! Host executable bridge for `dsl/std/termination.dag` `DescentEvidence` lattice
//! helpers while bootstrap still stages those `fn` bodies as
//! [`crate::dag::ArrowBody::Unparsed`].
//!
//! R3 gate `tier3_termination_mirror_dissolved`: [`crate::dag::DescentEvidence`]
//! remains the substrate carrier mirror in `dag.rs`, but lattice **operations**
//! are not duplicated there — they live here until std arrow bodies lower and
//! [`crate::evaluator::evaluate_body`] becomes the sole executable path.

use std::collections::HashMap;

use crate::dag::DescentEvidence;

pub fn evidence_rank(evidence: DescentEvidence) -> i64 {
    match evidence {
        DescentEvidence::Strict => 2,
        DescentEvidence::NonIncreasing => 1,
        DescentEvidence::DescentUnknown => 0,
    }
}

pub fn merge_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {
    match a {
        DescentEvidence::Strict => match b {
            DescentEvidence::Strict => DescentEvidence::Strict,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
        },
        DescentEvidence::NonIncreasing => match b {
            DescentEvidence::Strict => DescentEvidence::NonIncreasing,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
        },
        DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
    }
}

pub fn join_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {
    match a {
        DescentEvidence::DescentUnknown => b,
        DescentEvidence::NonIncreasing => match b {
            DescentEvidence::Strict => DescentEvidence::Strict,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::NonIncreasing,
        },
        DescentEvidence::Strict => DescentEvidence::Strict,
    }
}

/// Parity with `dsl/std/termination.dag::promote_to_strict` (fail-closed passthrough).
pub fn promote_to_strict(evidence: DescentEvidence) -> DescentEvidence {
    match evidence {
        DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
        DescentEvidence::Strict => DescentEvidence::Strict,
        DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
    }
}

pub fn optional_evidence_meet(
    a: Option<DescentEvidence>,
    b: Option<DescentEvidence>,
) -> Option<DescentEvidence> {
    match a {
        None => b,
        Some(va) => match b {
            None => a,
            Some(vb) => Some(merge_evidence(va, vb)),
        },
    }
}

pub fn map_evidence_merge_at(
    mut base: HashMap<String, DescentEvidence>,
    key: String,
    new_val: DescentEvidence,
) -> HashMap<String, DescentEvidence> {
    let merged = match base.get(&key).copied() {
        Some(existing) => merge_evidence(existing, new_val),
        None => new_val,
    };
    base.insert(key, merged);
    base
}
