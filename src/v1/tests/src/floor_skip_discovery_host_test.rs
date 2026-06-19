//! §5 execution receipts for Phase 1.5a floor skip host transport (`cli_run`).
//!
//! Proves the default (skip-disabled) corpus path runs without panic, skip-enabled
//! baseline miss fail-closes to running witnesses, and git diff observation failure
//! fail-closes to running the full explicit roster.

use std::path::PathBuf;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn workspace_root() -> PathBuf {
    crate::helpers::workspace_root()
}

fn floor_skip_test_roster() -> (Vec<String>, Vec<String>, Vec<(String, String)>) {
    let ws = workspace_root();
    let source_roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ];
    let entry = ws
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let function = "floor_runner_unaffected_verified_green_skips_holds".to_string();
    (source_roots, Vec::new(), vec![(entry, function)])
}

fn run_explicit_roster(skip: bool) -> Result<DiscoverySummary, String> {
    let (source_roots, scan_dirs, explicit) = floor_skip_test_roster();
    run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit,
        ExecutionMode::Wet,
        DiscoveryCorpusOptions {
            skip_unaffected_verified_baseline: skip,
        },
    )
}

#[test]
fn discovery_corpus_skip_disabled_runs_without_panic() {
    let summary = run_explicit_roster(false).expect("skip-disabled discovery must not panic");
    assert_eq!(summary.skipped, 0, "skip disabled → no baseline skips");
    assert!(
        summary.passed >= 1,
        "skip-disabled path must run at least one witness"
    );
}

#[test]
fn discovery_corpus_skip_enabled_baseline_miss_runs_corpus() {
    let _guard = baseline_cache_env_guard(None);
    let summary = run_explicit_roster(true).expect("baseline miss path must not panic");
    assert_eq!(
        summary.skipped, 0,
        "no cache → baseline miss → run witness (no skip)"
    );
    assert!(summary.passed >= 1);
}

#[test]
fn discovery_corpus_skip_enabled_git_observation_fail_closed_runs() {
    let _cache = baseline_cache_env_guard(None);
    let _base = env_var_guard("GUNBC_CI_DIFF_BASE", "__gunbc_invalid_diff_base__");
    let _head = env_var_guard("GUNBC_CI_DIFF_HEAD", "HEAD");
    let _merge = env_var_guard("GUNBC_CI_DIFF_MERGE_BASE", "0");
    let summary = run_explicit_roster(true).expect("git observation fail-closed must not panic");
    assert_eq!(
        summary.skipped, 0,
        "git diff failure → skip inactive → run full explicit roster"
    );
    assert!(summary.passed >= 1);
}

#[test]
fn discovery_corpus_skip_enabled_verified_green_baseline_skips_on_replay() {
    let cache = workspace_root().join(format!(
        "target/floor_skip_baseline_test_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&cache);
    std::fs::create_dir_all(&cache).expect("baseline cache dir");
    let _guard = baseline_cache_env_guard(Some(cache.as_path()));

    let first = run_explicit_roster(true).expect("first pass records baseline");
    assert_eq!(first.skipped, 0, "cold baseline → run witness once");
    assert!(first.passed >= 1);

    let second = run_explicit_roster(true).expect("replay must skip on verified-green baseline");
    assert_eq!(
        second.skipped, 1,
        "verified-green baseline tombstone → skip on replay"
    );

    let _ = std::fs::remove_dir_all(&cache);
}

struct EnvVarGuard {
    key: &'static str,
    prior: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prior = std::env::var_os(key);
        // SAFETY: test-only single-threaded env mutation.
        unsafe { std::env::set_var(key, value) };
        Self { key, prior }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn env_var_guard(key: &'static str, value: &str) -> EnvVarGuard {
    EnvVarGuard::set(key, value)
}

struct BaselineCacheEnvGuard {
    prior: Option<std::ffi::OsString>,
}

fn baseline_cache_env_guard(path: Option<&std::path::Path>) -> BaselineCacheEnvGuard {
    let prior = std::env::var_os("GUNBC_WITNESS_BASELINE_CACHE_DIR");
    match path {
        Some(p) => unsafe { std::env::set_var("GUNBC_WITNESS_BASELINE_CACHE_DIR", p) },
        None => unsafe { std::env::remove_var("GUNBC_WITNESS_BASELINE_CACHE_DIR") },
    }
    BaselineCacheEnvGuard { prior }
}

impl Drop for BaselineCacheEnvGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(v) => unsafe {
                std::env::set_var("GUNBC_WITNESS_BASELINE_CACHE_DIR", v)
            },
            None => unsafe { std::env::remove_var("GUNBC_WITNESS_BASELINE_CACHE_DIR") },
        }
    }
}
