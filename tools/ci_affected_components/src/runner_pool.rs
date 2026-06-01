//! Structural mirror of `SelfHostedRunnerPool` / `m1_probe_cargo_check_jobs_ceiling` in `src/v4/workflow/ci.dag`.
//! Lives outside `v3-compiler` (same crate as affected-set host transport).

/// Structural mirror of `SelfHostedRunnerPool` in `src/v4/workflow/ci.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfHostedRunnerPool {
    pub host: &'static str,
    pub arch: &'static str,
    pub core_count: u32,
    pub runner_count: u32,
    pub jobserver_token_cap: u32,
}

pub const CI_RUNNER_ARCH_ARM64: &str = "arm64";

pub const CI_SRV1_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv1",
    arch: CI_RUNNER_ARCH_ARM64,
    core_count: 128,
    runner_count: 20,
    jobserver_token_cap: 25,
};

pub const CI_SRV2_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv2",
    arch: CI_RUNNER_ARCH_ARM64,
    core_count: 128,
    runner_count: 30,
    jobserver_token_cap: 36,
};

pub const CI_SELF_HOSTED_RUNNER_POOLS: [SelfHostedRunnerPool; 2] = [CI_SRV1_POOL, CI_SRV2_POOL];

/// Governor ceiling handed to ctrl-build as `CTRL_BUILD_DYNAMIC_JOBS_MAX`
/// (`data m1_probe_cargo_check_jobs_ceiling` in `ci.dag`). Actual jobs are memory-denominated
/// at or below this; the MemAvailable term binds on a 128c/96GiB host.
pub const M1_PROBE_CARGO_CHECK_JOBS_CEILING: u32 = 64;

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

/// Modeled authority for `data m1_probe_cargo_check_jobs_ceiling` in `src/v4/workflow/ci.dag`.
pub fn m1_probe_cargo_check_jobs_ceiling() -> u32 {
    M1_PROBE_CARGO_CHECK_JOBS_CEILING
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srv_pools_match_operator_spec() {
        assert_eq!(CI_SRV1_POOL.arch, CI_RUNNER_ARCH_ARM64);
        assert_eq!(CI_SRV2_POOL.arch, CI_RUNNER_ARCH_ARM64);
        assert_eq!(CI_SRV1_POOL.runner_count, 20);
        assert_eq!(CI_SRV1_POOL.jobserver_token_cap, 25);
        assert_eq!(CI_SRV2_POOL.runner_count, 30);
        assert_eq!(CI_SRV2_POOL.jobserver_token_cap, 36);
    }

    #[test]
    fn m1_probe_cargo_check_jobs_is_operator_constant() {
        assert!(ci_runner_pool_fleet_capacities_valid());
        // Governor ceiling only — no static fallback (the probe fails closed without ctrl-build).
        // Actual job count is memory/pids-denominated by the host governor below this ceiling.
        assert_eq!(m1_probe_cargo_check_jobs_ceiling(), 64);
    }
}
