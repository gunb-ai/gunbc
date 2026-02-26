//! Transport fidelity ladder for tiered test generation.
//!
//! Each transport has a ladder of fidelity levels from pure mock (XS cost)
//! up to real remote calls (XL cost). Test generation emits variants at
//! each rung, gated by cost budget.

use crate::FermiCost;
use gunbc_ir::transport::TransportKind;

/// Fidelity level for transport testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FidelityLevel {
    /// XS: DryRun intercept, no real I/O.
    PureMock,
    /// S: In-memory hermetic (virtual filesystem, mock HTTP).
    VirtualIo,
    /// M: Real tempdir, sandboxed container.
    Sandboxed,
    /// L: Real local execution, unsandboxed.
    RealLocal,
    /// XL: Real network/remote calls.
    RealRemote,
}

impl FidelityLevel {
    /// Human-readable description.
    pub fn description(self) -> &'static str {
        match self {
            FidelityLevel::PureMock => "DryRun intercept (pure mock)",
            FidelityLevel::VirtualIo => "In-memory hermetic (virtual I/O)",
            FidelityLevel::Sandboxed => "Sandboxed (real tempdir/container)",
            FidelityLevel::RealLocal => "Real local (unsandboxed)",
            FidelityLevel::RealRemote => "Real remote (network)",
        }
    }
}

/// A single rung on the fidelity ladder.
#[derive(Debug, Clone)]
pub struct FidelityRung {
    /// Fermi cost of tests at this rung.
    pub cost: FermiCost,
    /// Fidelity level.
    pub level: FidelityLevel,
    /// Human-readable description.
    pub description: &'static str,
}

/// Fidelity ladder for a transport kind.
///
/// Lists the available test tiers from cheapest (PureMock) to most
/// realistic (RealRemote). Not all transports support all levels.
#[derive(Debug, Clone)]
pub struct FidelityLadder {
    /// Which transport this ladder applies to.
    pub transport: TransportKind,
    /// Available rungs, ordered cheapest-first.
    pub rungs: Vec<FidelityRung>,
}

impl FidelityLadder {
    /// Build a ladder for a specific transport kind.
    pub fn for_transport(kind: TransportKind) -> Self {
        match kind {
            TransportKind::File => Self {
                transport: kind,
                rungs: vec![
                    FidelityRung {
                        cost: FermiCost::XS,
                        level: FidelityLevel::PureMock,
                        description: "DryRun file intercept",
                    },
                    FidelityRung {
                        cost: FermiCost::S,
                        level: FidelityLevel::VirtualIo,
                        description: "In-memory virtual filesystem",
                    },
                    FidelityRung {
                        cost: FermiCost::M,
                        level: FidelityLevel::Sandboxed,
                        description: "Real tempdir with cleanup",
                    },
                    FidelityRung {
                        cost: FermiCost::L,
                        level: FidelityLevel::RealLocal,
                        description: "Real filesystem (no sandbox)",
                    },
                ],
            },
            TransportKind::Shell => Self {
                transport: kind,
                rungs: vec![
                    FidelityRung {
                        cost: FermiCost::XS,
                        level: FidelityLevel::PureMock,
                        description: "DryRun shell intercept",
                    },
                    FidelityRung {
                        cost: FermiCost::S,
                        level: FidelityLevel::VirtualIo,
                        description: "Scripted shell mock",
                    },
                    FidelityRung {
                        cost: FermiCost::M,
                        level: FidelityLevel::Sandboxed,
                        description: "Sandboxed shell (container)",
                    },
                    FidelityRung {
                        cost: FermiCost::L,
                        level: FidelityLevel::RealLocal,
                        description: "Real local shell",
                    },
                ],
            },
            TransportKind::Rest | TransportKind::Http => Self {
                transport: kind,
                rungs: vec![
                    FidelityRung {
                        cost: FermiCost::XS,
                        level: FidelityLevel::PureMock,
                        description: "DryRun HTTP/REST intercept",
                    },
                    FidelityRung {
                        cost: FermiCost::S,
                        level: FidelityLevel::VirtualIo,
                        description: "In-memory mock server",
                    },
                    FidelityRung {
                        cost: FermiCost::M,
                        level: FidelityLevel::Sandboxed,
                        description: "Local mock server (localhost)",
                    },
                    FidelityRung {
                        cost: FermiCost::XL,
                        level: FidelityLevel::RealRemote,
                        description: "Real remote API call",
                    },
                ],
            },
            TransportKind::Tcp => Self {
                transport: kind,
                rungs: vec![
                    FidelityRung {
                        cost: FermiCost::XS,
                        level: FidelityLevel::PureMock,
                        description: "DryRun TCP intercept",
                    },
                    FidelityRung {
                        cost: FermiCost::S,
                        level: FidelityLevel::VirtualIo,
                        description: "In-memory TCP mock",
                    },
                    FidelityRung {
                        cost: FermiCost::XL,
                        level: FidelityLevel::RealRemote,
                        description: "Real TCP connection",
                    },
                ],
            },
            TransportKind::LocalDirect => Self {
                transport: kind,
                rungs: vec![FidelityRung {
                    cost: FermiCost::XS,
                    level: FidelityLevel::PureMock,
                    description: "DryRun local-direct intercept",
                }],
            },
        }
    }

    /// Maximum fidelity level available in this ladder.
    pub fn max_level(&self) -> FidelityLevel {
        self.rungs
            .iter()
            .map(|r| r.level)
            .max()
            .unwrap_or(FidelityLevel::PureMock)
    }

    /// All rungs with cost at or below `max_cost`.
    pub fn rungs_up_to(&self, max_cost: FermiCost) -> Vec<&FidelityRung> {
        self.rungs.iter().filter(|r| r.cost <= max_cost).collect()
    }
}

/// Build canonical fidelity ladders for all transport kinds.
pub fn canonical_ladders() -> Vec<FidelityLadder> {
    vec![
        FidelityLadder::for_transport(TransportKind::File),
        FidelityLadder::for_transport(TransportKind::Shell),
        FidelityLadder::for_transport(TransportKind::Rest),
        FidelityLadder::for_transport(TransportKind::Http),
        FidelityLadder::for_transport(TransportKind::Tcp),
        FidelityLadder::for_transport(TransportKind::LocalDirect),
    ]
}

/// Compute the maximum fidelity achievable for a node given its transport deps.
///
/// For pure nodes (no transport deps), returns `PureMock` — always hermetic.
/// For nodes with transport deps, returns the *minimum* max-fidelity across
/// all dependencies (transitive meet).
pub fn node_max_fidelity(transport_deps: &[TransportKind]) -> FidelityLevel {
    if transport_deps.is_empty() {
        return FidelityLevel::PureMock;
    }
    transport_deps
        .iter()
        .map(|kind| FidelityLadder::for_transport(*kind).max_level())
        .min()
        .unwrap_or(FidelityLevel::PureMock)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_ladders_cover_all_transport_kinds() {
        let ladders = canonical_ladders();
        assert_eq!(ladders.len(), 6);
        assert!(ladders.iter().any(|l| l.transport == TransportKind::File));
        assert!(ladders.iter().any(|l| l.transport == TransportKind::Shell));
        assert!(ladders.iter().any(|l| l.transport == TransportKind::Rest));
        assert!(ladders.iter().any(|l| l.transport == TransportKind::Http));
        assert!(ladders.iter().any(|l| l.transport == TransportKind::Tcp));
        assert!(ladders
            .iter()
            .any(|l| l.transport == TransportKind::LocalDirect));
    }

    #[test]
    fn every_ladder_starts_with_pure_mock() {
        for ladder in canonical_ladders() {
            assert!(
                !ladder.rungs.is_empty(),
                "{:?} ladder has no rungs",
                ladder.transport
            );
            assert_eq!(
                ladder.rungs[0].level,
                FidelityLevel::PureMock,
                "{:?} ladder doesn't start with PureMock",
                ladder.transport
            );
        }
    }

    #[test]
    fn node_max_fidelity_pure_node() {
        assert_eq!(node_max_fidelity(&[]), FidelityLevel::PureMock);
    }

    #[test]
    fn node_max_fidelity_single_transport() {
        assert_eq!(
            node_max_fidelity(&[TransportKind::File]),
            FidelityLevel::RealLocal
        );
        assert_eq!(
            node_max_fidelity(&[TransportKind::Rest]),
            FidelityLevel::RealRemote
        );
    }

    #[test]
    fn node_max_fidelity_transitive_meet() {
        // File max = RealLocal (L), Rest max = RealRemote (XL)
        // Meet = min(RealLocal, RealRemote) = RealLocal
        assert_eq!(
            node_max_fidelity(&[TransportKind::File, TransportKind::Rest]),
            FidelityLevel::RealLocal
        );
    }

    #[test]
    fn node_max_fidelity_local_direct_constrains() {
        // LocalDirect max = PureMock
        assert_eq!(
            node_max_fidelity(&[TransportKind::Shell, TransportKind::LocalDirect]),
            FidelityLevel::PureMock
        );
    }

    #[test]
    fn rungs_up_to_filters_by_cost() {
        let ladder = FidelityLadder::for_transport(TransportKind::File);
        let xs_rungs = ladder.rungs_up_to(FermiCost::XS);
        assert_eq!(xs_rungs.len(), 1);
        assert_eq!(xs_rungs[0].level, FidelityLevel::PureMock);

        let s_rungs = ladder.rungs_up_to(FermiCost::S);
        assert_eq!(s_rungs.len(), 2);

        let all_rungs = ladder.rungs_up_to(FermiCost::XL);
        assert_eq!(all_rungs.len(), ladder.rungs.len());
    }

    #[test]
    fn fidelity_level_ordering() {
        assert!(FidelityLevel::PureMock < FidelityLevel::VirtualIo);
        assert!(FidelityLevel::VirtualIo < FidelityLevel::Sandboxed);
        assert!(FidelityLevel::Sandboxed < FidelityLevel::RealLocal);
        assert!(FidelityLevel::RealLocal < FidelityLevel::RealRemote);
    }
}
