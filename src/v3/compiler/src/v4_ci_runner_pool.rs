//! Structural mirror of `SelfHostedRunnerPool` / `m1_probe_cargo_check_jobs` in `src/v4/workflow/ci.dag`.

/// Structural mirror of `RunnerArch::Arm64` (single host arch today).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerArch {
    Arm64,
}

/// Structural mirror of `SelfHostedRunnerPool` in `src/v4/workflow/ci.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfHostedRunnerPool {
    pub host: &'static str,
    pub arch: RunnerArch,
    pub core_count: u32,
    pub runner_count: u32,
    pub jobserver_token_cap: u32,
}

pub const CI_SRV1_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv1",
    arch: RunnerArch::Arm64,
    core_count: 128,
    runner_count: 20,
    jobserver_token_cap: 25,
};

pub const CI_SRV2_POOL: SelfHostedRunnerPool = SelfHostedRunnerPool {
    host: "srv2",
    arch: RunnerArch::Arm64,
    core_count: 128,
    runner_count: 30,
    jobserver_token_cap: 36,
};

pub const CI_SELF_HOSTED_RUNNER_POOLS: [SelfHostedRunnerPool; 2] = [CI_SRV1_POOL, CI_SRV2_POOL];

pub fn ci_fleet_min_jobserver_token_cap() -> u32 {
    CI_SELF_HOSTED_RUNNER_POOLS
        .iter()
        .fold(CI_SRV1_POOL.jobserver_token_cap, |acc, pool| {
            acc.min(pool.jobserver_token_cap)
        })
}

pub fn ci_fleet_max_runner_count() -> u32 {
    CI_SELF_HOSTED_RUNNER_POOLS
        .iter()
        .fold(CI_SRV1_POOL.runner_count, |acc, pool| {
            acc.max(pool.runner_count)
        })
}

pub fn ci_fleet_min_runner_count() -> u32 {
    CI_SELF_HOSTED_RUNNER_POOLS
        .iter()
        .fold(CI_SRV1_POOL.runner_count, |acc, pool| {
            acc.min(pool.runner_count)
        })
}

pub fn ci_runner_pool_total_runner_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools.iter().map(|p| p.runner_count).sum()
}

pub fn ci_runner_pool_host_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools.len() as u32
}

pub fn ci_int_at_least_one(n: u32) -> u32 {
    n.max(1)
}

pub fn ci_runner_pool_m1_probe_witness_holds(pools: &[SelfHostedRunnerPool]) -> bool {
    !pools.is_empty() && ci_fleet_max_runner_count() != ci_fleet_min_runner_count()
}

pub fn ci_m1_probe_cargo_fanout_slots_from_fleet() -> Option<u32> {
    if !ci_runner_pool_m1_probe_witness_holds(&CI_SELF_HOSTED_RUNNER_POOLS) {
        return None;
    }
    let min_runners = ci_fleet_min_runner_count();
    let max_runners = ci_fleet_max_runner_count();
    let spread = max_runners - min_runners;
    if spread == 0 {
        return None;
    }
    let total_runners = ci_runner_pool_total_runner_count(&CI_SELF_HOSTED_RUNNER_POOLS);
    let hosts = ci_runner_pool_host_count(&CI_SELF_HOSTED_RUNNER_POOLS);
    Some((total_runners - min_runners) * hosts / spread)
}

pub fn ci_m1_probe_cargo_check_jobs_from_fleet() -> Option<u32> {
    let fanout = ci_m1_probe_cargo_fanout_slots_from_fleet()?;
    let fanout = ci_int_at_least_one(fanout);
    if fanout == 0 {
        return None;
    }
    Some(ci_fleet_min_jobserver_token_cap() / fanout)
}

/// Modeled authority for `data m1_probe_cargo_check_jobs` in `src/v4/workflow/ci.dag`.
pub fn m1_probe_cargo_check_jobs() -> u32 {
    ci_m1_probe_cargo_check_jobs_from_fleet().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srv_pools_match_operator_spec() {
        assert_eq!(CI_SRV1_POOL.runner_count, 20);
        assert_eq!(CI_SRV1_POOL.jobserver_token_cap, 25);
        assert_eq!(CI_SRV2_POOL.runner_count, 30);
        assert_eq!(CI_SRV2_POOL.jobserver_token_cap, 36);
    }

    #[test]
    fn m1_probe_cargo_check_jobs_derived_as_four() {
        assert_eq!(ci_m1_probe_cargo_fanout_slots_from_fleet(), Some(6));
        assert_eq!(ci_m1_probe_cargo_check_jobs_from_fleet(), Some(4));
        assert_eq!(m1_probe_cargo_check_jobs(), 4);
    }

    #[test]
    fn witness_required_for_fleet_m1_derivation() {
        assert!(ci_runner_pool_m1_probe_witness_holds(
            &CI_SELF_HOSTED_RUNNER_POOLS
        ));
    }
}
