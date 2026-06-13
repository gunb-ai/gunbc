//! CI component affected-set — always-supported CI host transport.
//!
//! **Authority:** `.github/workflows/ci.yml` (hand-edited CI source of truth; `ci_component_affected`
//! path predicates and `ci_changed_path_affects_*` buckets). This crate is the Rust mirror those
//! predicates execute in the affected job. The former `src/v4/workflow/ci.dag` ↔ mirror set-equality
//! ratchet (`tools/ci_workflow_ratchet`) retired with #4543; keep aligned via behavioral fixture
//! tests in this crate. Lives outside `v3-compiler` so the affected job can emit `v3=false` without
//! compiling the frozen v3 package first (INVARIANTS P3 fail-closed boundary).

pub mod git_diff_transport;
pub mod receipt;
pub mod runner_pool;
pub mod wave3_shadow_receipt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CiComponentAffected {
    pub v2: bool,
    pub v3: bool,
    pub v4: bool,
    pub testclaim_corpus: bool,
    pub workflow_policy: bool,
    pub release_distribution: bool,
    /// True when every changed path that triggers any CI component bucket is a
    /// release-distribution path and at least one such path is present.
    pub release_distribution_only: bool,
}

/// Mirror of `ci_component_affected_fail_closed` — all components affected (INVARIANTS P3).
pub fn ci_component_affected_fail_closed() -> CiComponentAffected {
    CiComponentAffected {
        v2: true,
        v3: true,
        v4: true,
        testclaim_corpus: true,
        workflow_policy: true,
        release_distribution: true,
        release_distribution_only: false,
    }
}

/// Map `git diff --name-only` paths to component flags (same semantics as ci.yml affected-set gating).
pub fn ci_component_affected_from_changed_paths<'a, I>(changed: I) -> CiComponentAffected
where
    I: IntoIterator<Item = &'a str>,
{
    let mut out = CiComponentAffected {
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
        release_distribution_only: false,
    };
    let changed: Vec<&str> = changed.into_iter().collect();
    for path in &changed {
        if ci_changed_path_affects_v2(path) {
            out.v2 = true;
        }
        if ci_changed_path_affects_v3(path) {
            out.v3 = true;
        }
        if ci_changed_path_affects_v4(path) {
            out.v4 = true;
        }
        if ci_changed_path_affects_testclaim_corpus(path) {
            out.testclaim_corpus = true;
        }
        if ci_changed_path_affects_workflow_policy(path) {
            out.workflow_policy = true;
        }
        if ci_changed_path_affects_release_distribution(path) {
            out.release_distribution = true;
        }
    }
    out.release_distribution_only =
        ci_release_distribution_only_from_changed_paths(changed.iter().copied());
    out
}

fn ci_changed_path_triggers_ci_component(path: &str) -> bool {
    ci_changed_path_affects_v2(path)
        || ci_changed_path_affects_v3(path)
        || ci_changed_path_affects_v4(path)
        || ci_changed_path_affects_testclaim_corpus(path)
        || ci_changed_path_affects_workflow_policy(path)
        || ci_changed_path_affects_release_distribution(path)
}

/// RELEASE §5 — skip orthogonal fixture gates only when the diff is exclusively
/// release-distribution paths (mixed release + phase1 fixture paths must not skip).
pub fn ci_release_distribution_only_from_changed_paths<'a, I>(changed: I) -> bool
where
    I: IntoIterator<Item = &'a str>,
{
    let paths: Vec<&str> = changed.into_iter().collect();
    let mut saw_release_path = false;
    for path in &paths {
        if ci_changed_path_affects_release_distribution(path) {
            saw_release_path = true;
        }
        if ci_changed_path_triggers_ci_component(path)
            && !ci_changed_path_affects_release_distribution(path)
        {
            return false;
        }
    }
    saw_release_path
}

pub fn ci_changed_path_affects_v2(path: &str) -> bool {
    path.starts_with("src/v2/") || path == "Cargo.toml" || path == "Cargo.lock"
}

pub fn ci_changed_path_affects_v3(path: &str) -> bool {
    path.starts_with("src/v3/")
        || path.starts_with("dsl/")
        || path == "Cargo.toml"
        || path == "Cargo.lock"
}

pub fn ci_changed_path_affects_v4(path: &str) -> bool {
    path == "src/v4/bin/main.dag"
        || path == "src/v4/program.dag"
        || path.starts_with("src/v4/program/")
        || path.starts_with("src/v4/workflow/")
        || path.starts_with("src/v4/compiler/")
        || path.starts_with("src/v4/std/")
        || path.starts_with("src/v4/extdeps/")
        || path.starts_with("src/v4/lens/")
        || path.starts_with("src/v4/test/claim/manual/")
        || path.starts_with("src/v4/test/claim/generated/")
        || path.starts_with("src/v4/test/fixture/")
        || path.starts_with("src/v4/test/v2_run_preflight/")
        || path == "src/v4/test/coercion_fold_int_rust_fixture.dag"
        || path.starts_with("fixtures/v4-mvp1/")
        || path == ".github/ci-floor/v4-rust-full-tree-emit-probe.sh"
        || path.starts_with(".github/ci-floor/")
        || path.starts_with("dsl/std/")
        || path == "Cargo.toml"
        || path == "Cargo.lock"
}

pub fn ci_changed_path_affects_testclaim_corpus(path: &str) -> bool {
    ci_changed_path_affects_v4(path)
        || path.starts_with("src/v4/test/claim/")
        || path == "scripts/v4-testclaim-corpus-eval.sh"
        || path == "scripts/v4-testclaim-smoke-roster.sh"
        || path == "scripts/v4-discover-owned-data.sh"
        || path == "scripts/v4-substrate-equivalence-gate.sh"
        || path.starts_with("scripts/fixtures/v4_discovery_completeness_slice/")
        || path == "src/v2/stage0/Cargo.toml"
        || path == "src/v2/stage0/src/bin/discover_owned_data.rs"
        || path == "src/v2/stage0/src/cli_run.rs"
        || path == "src/v4/test/claim/workflow/discovery_types.dag"
        || path == "src/v4/test/claim/workflow/glob_discovery.dag"
        || path == "src/v4/test/claim/workflow/glob_discovery_law.dag"
        || path == "src/v4/test/claim/workflow/host_discovered_owned_data_manifest.dag"
        || path == "src/v4/test/claim/workflow/unified_test_claim_substrate_equivalence.dag"
}

pub fn ci_changed_path_affects_workflow_policy(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || path.starts_with("src/v4/workflow/ci/")
        || path.starts_with("tools/ci_affected_components")
        || path.starts_with(".github/ci-floor/")
}

pub fn ci_changed_path_affects_release_distribution(path: &str) -> bool {
    path == "src/v4/workflow/release.dag"
        || path == ".github/workflows/release.yml"
        || path == "install.sh"
        || path == "install/release-target-triples.sh"
        || path == "src/v4/install/install.dag"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_workspace_deps_without_src_v2() {
        assert!(ci_changed_path_affects_v2("Cargo.lock"));
        assert!(ci_changed_path_affects_v3("Cargo.lock"));
        assert!(ci_changed_path_affects_v4("Cargo.lock"));
    }

    #[test]
    fn v3_tier0_includes_workspace_deps_for_i3() {
        assert!(ci_changed_path_affects_v3("Cargo.toml"));
        assert!(ci_changed_path_affects_v3("Cargo.lock"));
        assert!(ci_changed_path_affects_v3("src/v3/compiler/src/lib.rs"));
        assert!(ci_changed_path_affects_v3("dsl/gunbc/ci.dag"));
        assert!(ci_changed_path_affects_v3("dsl/std/node.dag"));
    }

    #[test]
    fn v4_m1_probe_script_triggers_v4_bucket() {
        assert!(ci_changed_path_affects_v4(
            ".github/ci-floor/v4-rust-full-tree-emit-probe.sh"
        ));
        assert!(ci_changed_path_affects_workflow_policy(
            ".github/ci-floor/v4-bootstrap-viability.sh"
        ));
    }

    #[test]
    fn v4_bucket_uses_gate_frontier_not_all_src_v4() {
        assert!(ci_changed_path_affects_v4("src/v4/std/node.dag"));
        assert!(ci_changed_path_affects_v4("src/v4/compiler/05_eval.dag"));
        assert!(ci_changed_path_affects_v4(
            "src/v4/test/claim/manual/mvp1_rust_add_translate.dag"
        ));
        assert!(!ci_changed_path_affects_v4(
            "src/v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag"
        ));
        assert!(!ci_changed_path_affects_v4(
            "src/v4/test/claim/workflow/affected_set_ci_runner.dag"
        ));
        // Previously tripped v4 via blanket `src/v4/`; now release_distribution or testclaim only.
        assert!(!ci_changed_path_affects_v4("src/v4/install/install.dag"));
        assert!(ci_changed_path_affects_release_distribution(
            "src/v4/install/install.dag"
        ));
        assert!(!ci_changed_path_affects_v4("src/v4/TASKS.md"));
    }

    #[test]
    fn gram_emit_substrate_triggers_testclaim_corpus() {
        assert!(ci_changed_path_affects_v4("src/v4/std/grammar.dag"));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/std/grammar.dag"
        ));
        assert!(!ci_changed_path_affects_workflow_policy(
            "src/v4/std/grammar.dag"
        ));
    }

    #[test]
    fn testclaim_corpus_includes_all_v4_claim_paths() {
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/claim/workflow/affected_set_ci_runner.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/claim/manual/mvp1_rust_add_translate.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/claim/lens_parallelism/data_dependency.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/std/grammar.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "scripts/v4-testclaim-corpus-eval.sh"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "scripts/v4-testclaim-smoke-roster.sh"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "scripts/v4-discover-owned-data.sh"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "scripts/v4-substrate-equivalence-gate.sh"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "scripts/fixtures/v4_discovery_completeness_slice/edit_locus_resolver.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v2/stage0/Cargo.toml"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v2/stage0/src/bin/discover_owned_data.rs"
        ));
        assert!(!ci_changed_path_affects_testclaim_corpus(
            "scripts/other.sh"
        ));
    }

    #[test]
    fn dsl_std_triggers_v3_and_v4() {
        assert!(ci_changed_path_affects_v3("dsl/std/node.dag"));
        assert!(ci_changed_path_affects_v4("dsl/std/node.dag"));
    }

    #[test]
    fn workflow_policy_includes_ci_yml_authority() {
        assert!(ci_changed_path_affects_workflow_policy(
            ".github/workflows/ci.yml"
        ));
        assert!(ci_changed_path_affects_workflow_policy(
            "tools/ci_affected_components/src/runner_pool.rs"
        ));
    }

    #[test]
    fn aggregate_fixture_matches_modeled_buckets() {
        let flags = ci_component_affected_from_changed_paths([
            ".github/ci-floor/v4-rust-full-tree-emit-probe.sh",
            "docs/unrelated.md",
        ]);
        assert!(!flags.v2);
        assert!(!flags.v3);
        assert!(flags.v4);
        assert!(flags.testclaim_corpus);
        assert!(flags.workflow_policy);
        assert!(!flags.release_distribution);
    }

    #[test]
    fn claim_corpus_paths_only_raise_testclaim_bucket() {
        let flags = ci_component_affected_from_changed_paths([
            "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
        ]);
        assert!(!flags.v2);
        assert!(!flags.v3);
        assert!(!flags.v4);
        assert!(flags.testclaim_corpus);
        assert!(!flags.workflow_policy);
        assert!(!flags.release_distribution);
    }

    #[test]
    fn v4_compile_harness_paths_inherit_testclaim_corpus_via_affects_v4() {
        assert!(ci_changed_path_affects_v4(
            "src/v4/test/coercion_fold_int_rust_fixture.dag"
        ));
        assert!(ci_changed_path_affects_v4(
            "src/v4/test/v2_run_preflight/MOVE1_COVERAGE.txt"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/coercion_fold_int_rust_fixture.dag"
        ));
        assert!(ci_changed_path_affects_testclaim_corpus(
            "src/v4/test/v2_run_preflight/MOVE1_COVERAGE.txt"
        ));
    }

    #[test]
    fn triggers_ci_component_includes_testclaim_corpus() {
        assert!(ci_changed_path_triggers_ci_component(
            "src/v4/test/claim/workflow/affected_set_ci_runner.dag"
        ));
    }

    #[test]
    fn release_distribution_only_excludes_mixed_fixture_paths() {
        assert!(ci_release_distribution_only_from_changed_paths([
            "install.sh",
            "src/v4/install/install.dag",
        ]));
        assert!(!ci_release_distribution_only_from_changed_paths([
            "install.sh",
            ".github/ci-floor/v4-rust-full-tree-emit-probe.sh",
        ]));
        assert!(!ci_release_distribution_only_from_changed_paths([
            "install.sh",
            "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
        ]));
        assert!(!ci_release_distribution_only_from_changed_paths([
            "src/v4/install/install.dag",
            ".github/workflows/ci.yml",
        ]));
        assert!(!ci_release_distribution_only_from_changed_paths([
            "docs/README.md"
        ]));
    }

    #[test]
    fn affects_v4_covers_full_ci_floor_compile_closure() {
        // program.dag and program/ subdirectory are in the ci_floor compile closure
        assert!(ci_changed_path_affects_v4("src/v4/program.dag"));
        assert!(ci_changed_path_affects_v4("src/v4/program/program.dag"));
        // workflow/ files (runtime_run.dag and lens_ci_gate.dag) are in the compile closure
        assert!(ci_changed_path_affects_v4(
            "src/v4/workflow/runtime_run.dag"
        ));
        assert!(ci_changed_path_affects_v4(
            "src/v4/workflow/lens_ci_gate.dag"
        ));
        // bootstrap.dag still covered by the prefix
        assert!(ci_changed_path_affects_v4("src/v4/workflow/bootstrap.dag"));
    }

    #[test]
    fn release_distribution_includes_release_authority_paths() {
        assert!(ci_changed_path_affects_release_distribution(
            "src/v4/workflow/release.dag"
        ));
        assert!(ci_changed_path_affects_release_distribution("install.sh"));
        assert!(ci_changed_path_affects_release_distribution(
            "install/release-target-triples.sh"
        ));
        assert!(ci_changed_path_affects_release_distribution(
            "src/v4/install/install.dag"
        ));
        assert!(!ci_changed_path_affects_release_distribution(
            ".github/workflows/ci.yml"
        ));
    }
}
