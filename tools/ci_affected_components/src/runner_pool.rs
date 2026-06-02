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

/// Structural mirror of `RunnerPoolResiliencePhase` in `src/v4/workflow/ci.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerPoolResiliencePhase {
    Detect,
    Fallback,
    Comms,
    Recovery,
}

/// Structural mirror of `RunnerPoolResilienceStep` in `src/v4/workflow/ci.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerPoolResilienceStep {
    pub phase: RunnerPoolResiliencePhase,
    pub trigger: &'static str,
    pub action: &'static str,
    pub owner: &'static str,
    pub fail_closed: bool,
}

pub const CI_RUNNER_POOL_RESILIENCE_PLAYBOOK: [RunnerPoolResilienceStep; 4] = [
    RunnerPoolResilienceStep {
        phase: RunnerPoolResiliencePhase::Detect,
        trigger: "runner_pool_jobserver_coupling_lost",
        action: "runner_pool_fail_required_path",
        owner: "ci_manager_owner",
        fail_closed: true,
    },
    RunnerPoolResilienceStep {
        phase: RunnerPoolResiliencePhase::Fallback,
        trigger: "runner_pool_ci_required_path_at_risk",
        action: "runner_pool_route_to_github_hosted_floor",
        owner: "ci_manager_owner",
        fail_closed: true,
    },
    RunnerPoolResilienceStep {
        phase: RunnerPoolResiliencePhase::Comms,
        trigger: "runner_pool_operator_status_change_required",
        action: "runner_pool_notify_parent_dashboard",
        owner: "ci_manager_owner",
        fail_closed: true,
    },
    RunnerPoolResilienceStep {
        phase: RunnerPoolResiliencePhase::Recovery,
        trigger: "runner_pool_capacity_restored",
        action: "runner_pool_reenable_self_hosted_capacity",
        owner: "ci_operator_owner",
        fail_closed: true,
    },
];

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

pub fn ci_runner_pool_resilience_playbook_fail_closed() -> bool {
    CI_RUNNER_POOL_RESILIENCE_PLAYBOOK
        .iter()
        .all(|step| step.fail_closed)
}

pub fn ci_runner_pool_resilience_playbook_has_phase(phase: RunnerPoolResiliencePhase) -> bool {
    CI_RUNNER_POOL_RESILIENCE_PLAYBOOK
        .iter()
        .any(|step| step.phase == phase)
}

pub fn ci_runner_pool_resilience_playbook_complete() -> bool {
    ci_runner_pool_resilience_playbook_fail_closed()
        && ci_runner_pool_resilience_playbook_has_phase(RunnerPoolResiliencePhase::Detect)
        && ci_runner_pool_resilience_playbook_has_phase(RunnerPoolResiliencePhase::Fallback)
        && ci_runner_pool_resilience_playbook_has_phase(RunnerPoolResiliencePhase::Comms)
        && ci_runner_pool_resilience_playbook_has_phase(RunnerPoolResiliencePhase::Recovery)
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

    #[test]
    fn runner_pool_resilience_playbook_covers_required_phases() {
        assert!(ci_runner_pool_resilience_playbook_complete());
    }

    #[test]
    fn runner_pool_resilience_playbook_is_fail_closed() {
        assert!(ci_runner_pool_resilience_playbook_fail_closed());
    }
}
