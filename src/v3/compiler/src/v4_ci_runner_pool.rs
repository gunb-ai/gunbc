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

pub const CI_SELF_HOSTED_RUNNER_POOLS: [SelfHostedRunnerPool; 2] =
    [CI_SRV1_POOL, CI_SRV2_POOL];

pub fn ci_runner_pool_min_jobserver_token_cap(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools
        .iter()
        .map(|p| p.jobserver_token_cap)
        .min()
        .unwrap_or(0)
}

pub fn ci_runner_pool_max_runner_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools.iter().map(|p| p.runner_count).max().unwrap_or(0)
}

pub fn ci_runner_pool_min_runner_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools
        .iter()
        .map(|p| p.runner_count)
        .min()
        .unwrap_or(0)
}

pub fn ci_runner_pool_total_runner_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools.iter().map(|p| p.runner_count).sum()
}

pub fn ci_runner_pool_host_count(pools: &[SelfHostedRunnerPool]) -> u32 {
    pools.len() as u32
}

pub fn ci_runner_pool_runner_spread(max_runners: u32, min_runners: u32) -> u32 {
    if max_runners == min_runners {
        1
    } else {
        max_runners - min_runners
    }
}

pub fn ci_m1_probe_cargo_fanout_slots(pools: &[SelfHostedRunnerPool]) -> u32 {
    let min_runners = ci_runner_pool_min_runner_count(pools);
    let max_runners = ci_runner_pool_max_runner_count(pools);
    let total_runners = ci_runner_pool_total_runner_count(pools);
    let hosts = ci_runner_pool_host_count(pools);
    let spread = ci_runner_pool_runner_spread(max_runners, min_runners);
    (total_runners - min_runners) * hosts / spread
}

pub fn ci_m1_probe_cargo_check_jobs_from_pools(pools: &[SelfHostedRunnerPool]) -> u32 {
    let fanout = ci_m1_probe_cargo_fanout_slots(pools).max(1);
    ci_runner_pool_min_jobserver_token_cap(pools) / fanout
}

/// Modeled authority for `data m1_probe_cargo_check_jobs` in `src/v4/workflow/ci.dag`.
pub fn m1_probe_cargo_check_jobs() -> u32 {
    ci_m1_probe_cargo_check_jobs_from_pools(&CI_SELF_HOSTED_RUNNER_POOLS)
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
        assert_eq!(ci_m1_probe_cargo_fanout_slots(&CI_SELF_HOSTED_RUNNER_POOLS), 6);
        assert_eq!(m1_probe_cargo_check_jobs(), 4);
    }
}
