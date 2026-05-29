//! v4 `workflow/ci.dag` component affected-set — always-supported CI host transport.
//!
//! **Modeled authority:** `src/v4/workflow/ci.dag` (`ci_component_affected_from_git_diff` and
//! `ci_changed_path_affects_*`). This crate is an interim Rust mirror of those predicates for
//! the CI runner (not `.dag` eval yet); keep aligned via `v4_workflow_ci_runner_dag_smoke_test`
//! set-equality + behavioral fixture parity (not substring presence). Lives outside `v3-compiler`
//! so the affected job can emit `v3=false` without compiling the frozen v3 package first
//! (INVARIANTS P3 fail-closed boundary).

pub mod runner_pool;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CiComponentAffected {
    pub v2: bool,
    pub v3: bool,
    pub v4: bool,
    pub workflow_policy: bool,
}

/// Mirror of `ci_component_affected_fail_closed` — all components affected (INVARIANTS P3).
pub fn ci_component_affected_fail_closed() -> CiComponentAffected {
    CiComponentAffected {
        v2: true,
        v3: true,
        v4: true,
        workflow_policy: true,
    }
}

/// Map `git diff --name-only` paths to component flags (same semantics as `ci.dag`).
pub fn ci_component_affected_from_changed_paths<'a, I>(changed: I) -> CiComponentAffected
where
    I: IntoIterator<Item = &'a str>,
{
    let mut out = CiComponentAffected {
        v2: false,
        v3: false,
        v4: false,
        workflow_policy: false,
    };
    for path in changed {
        if ci_changed_path_affects_v2(path) {
            out.v2 = true;
        }
        if ci_changed_path_affects_v3(path) {
            out.v3 = true;
        }
        if ci_changed_path_affects_v4(path) {
            out.v4 = true;
        }
        if ci_changed_path_affects_workflow_policy(path) {
            out.workflow_policy = true;
        }
    }
    out
}

pub fn ci_changed_path_affects_v2(path: &str) -> bool {
    path.starts_with("src/v2/") || path == "Cargo.toml" || path == "Cargo.lock"
}

pub fn ci_changed_path_affects_v3(path: &str) -> bool {
    path.starts_with("src/v3/") || path.starts_with("dsl/")
}

pub fn ci_changed_path_affects_v4(path: &str) -> bool {
    path.starts_with("src/v4/")
        || path.starts_with("fixtures/v4-mvp1/")
        || path == "scripts/v4-mvp1-e2e-gate.sh"
        || path == "scripts/v4-m1-rust-emit-probe.sh"
        || path.starts_with("scripts/v4-mvp1")
        || path.starts_with("scripts/v4-m1")
        || path.starts_with("scripts/v4-testclaim-")
        || path.starts_with("dsl/std/")
        || path == "Cargo.toml"
        || path == "Cargo.lock"
}

pub fn ci_changed_path_affects_workflow_policy(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || path.starts_with("src/v4/workflow/ci/")
        || path.starts_with("tools/ci_affected_components")
        || path.starts_with("scripts/check-workflow-path-regex-inventory")
        || path.starts_with("scripts/workflow-path-regex-forbidden-substrings")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_workspace_deps_without_src_v2() {
        assert!(ci_changed_path_affects_v2("Cargo.lock"));
        assert!(!ci_changed_path_affects_v3("Cargo.lock"));
        assert!(ci_changed_path_affects_v4("Cargo.lock"));
    }

    #[test]
    fn v3_freeze_ignores_cargo_toml_for_v3_only() {
        assert!(!ci_changed_path_affects_v3("Cargo.toml"));
        assert!(ci_changed_path_affects_v3("src/v3/compiler/src/lib.rs"));
        assert!(ci_changed_path_affects_v3("dsl/gunbc/ci.dag"));
        assert!(ci_changed_path_affects_v3("dsl/std/node.dag"));
    }

    #[test]
    fn v4_m1_probe_script_triggers_v4_bucket() {
        assert!(ci_changed_path_affects_v4(
            "scripts/v4-m1-rust-emit-probe.sh"
        ));
        assert!(ci_changed_path_affects_v4("scripts/v4-mvp1-e2e-gate.sh"));
    }

    #[test]
    fn dsl_std_triggers_v3_and_v4() {
        assert!(ci_changed_path_affects_v3("dsl/std/node.dag"));
        assert!(ci_changed_path_affects_v4("dsl/std/node.dag"));
    }

    #[test]
    fn workflow_policy_includes_modeled_ci_authority() {
        assert!(ci_changed_path_affects_workflow_policy(
            "src/v4/workflow/ci.dag"
        ));
        assert!(ci_changed_path_affects_workflow_policy(
            ".github/workflows/ci.yml"
        ));
        assert!(ci_changed_path_affects_workflow_policy(
            "tools/ci_affected_components/src/runner_pool.rs"
        ));
    }

    #[test]
    fn aggregate_fixture_matches_shell_script_buckets() {
        let flags = ci_component_affected_from_changed_paths([
            "src/v4/workflow/ci.dag",
            "docs/unrelated.md",
        ]);
        assert!(!flags.v2);
        assert!(!flags.v3);
        assert!(flags.v4);
        assert!(flags.workflow_policy);
    }
}
