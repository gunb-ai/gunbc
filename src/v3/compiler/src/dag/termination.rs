//! `std.termination` mirror for Lane E-T.
//!
//! Structural carriers aligned with `src/v3/std/termination.dag`. The v3
//! bootstrap carries the `.dag` declarations; this Rust mirror gives tests and
//! transitional compiler consumers executable access to the same lattice until
//! std function bodies are emitted from `.dag` directly.

use std::collections::HashMap;

/// Descent evidence lattice: `DescentUnknown < NonIncreasing < Strict`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescentEvidence {
    Strict,
    NonIncreasing,
    DescentUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RankingDimension {
    TreeSize { param: String },
    ListLength { param: String },
    ArithmeticValue { param: String },
    TokenPosition { param: String },
    SetCardinality { param: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescentSource {
    ChildAccessor { accessor: String },
    ListShrink { amount: i64 },
    ArithmeticDecrease { op: String, by: i64 },
    ParserAdvance { witness: String },
    SetRemoval { element: String },
    FoldIteration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminationProof {
    pub dimensions: Vec<RankingDimension>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofEdge {
    pub caller: String,
    pub callee: String,
    pub evidence: Vec<DescentEvidence>,
}

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

pub fn promote_to_strict(evidence: DescentEvidence) -> DescentEvidence {
    match evidence {
        DescentEvidence::NonIncreasing => DescentEvidence::Strict,
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
