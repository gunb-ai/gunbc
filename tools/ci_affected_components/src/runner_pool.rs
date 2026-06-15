//! Host transport projection of the operator-fleet CI runner pool.
//!
//! **Authority:** `dsl/std/compute_fabric.dag` (`CiRunnerPoolFacts`, `supply_srv{1,2}_ci_runner_pool`,
//! `supply_srv{1,2}_offer.constraints`). This module is the Rust host mirror for tools that have
//! not yet routed gate-3 dispatch through compute-fabric eval (CF-M1). Dissolve when the
//! one-binary CI runner consumes `gunbc.tools.ci_runner_pool` via v2 eval instead of these
//! constants.
//!
//! Parity witness: `dsl/test/claim/ci_runner_pool_compute_fabric_projection.dag`.

/// Host-side mirror of `std.compute_fabric::CiRunnerPoolFacts` for srv1/srv2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfHostedRunnerPool {
    pub host: &'static str,
    pub arch: &'static str,
    pub core_count: u32,
    pub runner_count: u32,
    pub jobserver_token_cap: u32,
}

pub const CI_RUNNER_ARCH_ARM64: &str = "arm64";

/// Projected from `supply_srv1_ci_runner_pool` in `dsl/std/compute_fabric.dag`.
pub const CI_SRV1_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv1",
    arch: CI_RUNNER_ARCH_ARM64,
    core_count: 128,
    runner_count: 20,
    jobserver_token_cap: 25,
};

/// Projected from `supply_srv2_ci_runner_pool` in `dsl/std/compute_fabric.dag`.
pub const CI_SRV2_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv2",
    arch: CI_RUNNER_ARCH_ARM64,
    core_count: 128,
    runner_count: 30,
    jobserver_token_cap: 36,
};

pub const CI_SELF_HOSTED_RUNNER_POOLS: [SelfHostedRunnerPool; 2] = [CI_SRV1_POOL, CI_SRV2_POOL];

pub fn ci_int_positive(n: u32) -> bool {
    n >= 1
}

pub fn ci_runner_pool_capacity_valid(pool: SelfHostedRunnerPool) -> bool {
    ci_int_positive(pool.core_count)
        && ci_int_positive(pool.runner_count)
        && ci_int_positive(pool.jobserver_token_cap)
}

pub fn ci_runner_pool_fleet_capacities_valid() -> bool {
    CI_SELF_HOSTED_RUNNER_POOLS
        .iter()
        .all(|pool| ci_runner_pool_capacity_valid(*pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srv_pools_match_compute_fabric_supply_projection() {
        // Numeric facts must stay aligned with supply_srv{1,2}_ci_runner_pool in compute_fabric.dag.
        assert_eq!(CI_SRV1_POOL.arch, CI_RUNNER_ARCH_ARM64);
        assert_eq!(CI_SRV2_POOL.arch, CI_RUNNER_ARCH_ARM64);
        assert_eq!(CI_SRV1_POOL.runner_count, 20);
        assert_eq!(CI_SRV1_POOL.jobserver_token_cap, 25);
        assert_eq!(CI_SRV2_POOL.runner_count, 30);
        assert_eq!(CI_SRV2_POOL.jobserver_token_cap, 36);
        assert_eq!(CI_SRV1_POOL.core_count, 128);
        assert_eq!(CI_SRV2_POOL.core_count, 128);
    }

    #[test]
    fn fleet_capacities_valid() {
        assert!(ci_runner_pool_fleet_capacities_valid());
    }
}
