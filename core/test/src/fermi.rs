//! Test classification + cost gating helpers.
//!
//! This module provides a lightweight model for test class (unit/hermetic/integration)
//! and a Fermi-style cost budget (XS..XL). Tests can opt in to gating via `guard()`.

use std::env;

/// Test classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestClass {
    Unit,
    Hermetic,
    Integration,
}

impl TestClass {
    pub fn as_str(self) -> &'static str {
        match self {
            TestClass::Unit => "unit",
            TestClass::Hermetic => "hermetic",
            TestClass::Integration => "integration",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "unit" => Some(TestClass::Unit),
            "hermetic" => Some(TestClass::Hermetic),
            "integration" => Some(TestClass::Integration),
            _ => None,
        }
    }
}

/// Fermi-style test cost bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FermiCost {
    XS,
    S,
    M,
    L,
    XL,
}

impl FermiCost {
    pub fn as_str(self) -> &'static str {
        match self {
            FermiCost::XS => "XS",
            FermiCost::S => "S",
            FermiCost::M => "M",
            FermiCost::L => "L",
            FermiCost::XL => "XL",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_uppercase().as_str() {
            "XS" => Some(FermiCost::XS),
            "S" => Some(FermiCost::S),
            "M" => Some(FermiCost::M),
            "L" => Some(FermiCost::L),
            "XL" => Some(FermiCost::XL),
            _ => None,
        }
    }

    /// Fermi-estimated timeout in milliseconds for this cost bucket.
    ///
    /// Used as the default shell command timeout when callers don't specify
    /// an explicit value. Also used for CI job timeout estimation.
    pub fn timeout_ms(self) -> u64 {
        match self {
            FermiCost::XS => 30_000,      //  30 s
            FermiCost::S => 300_000,       //   5 min
            FermiCost::M => 600_000,       //  10 min
            FermiCost::L => 1_800_000,     //  30 min
            FermiCost::XL => 3_600_000,    //  60 min
        }
    }
}

/// Metadata for a test case (used for gating decisions).
#[derive(Debug, Clone)]
pub struct TestMeta<'a> {
    pub name: &'a str,
    pub class: TestClass,
    pub cost: FermiCost,
    pub requires: &'a [&'a str],
    pub secrets: &'a [&'a str],
}

/// Determine the max cost allowed for tests.
pub fn max_cost_from_env() -> FermiCost {
    env::var("GUNBC_TEST_MAX_COST")
        .ok()
        .and_then(|v| FermiCost::parse(&v))
        .unwrap_or_else(|| {
            if in_github_actions() {
                FermiCost::XL
            } else {
                FermiCost::S
            }
        })
}

/// True when `GUNBC_TEST_MAX_COST` is explicitly set in the environment.
///
/// When the cost limit is explicit, guards should skip silently instead of
/// panicking — the caller made a deliberate choice (e.g., the preflight
/// test gate limits to `S` to avoid running live tests that require secrets).
fn cost_limit_is_explicit() -> bool {
    env::var("GUNBC_TEST_MAX_COST")
        .ok()
        .and_then(|v| FermiCost::parse(&v))
        .is_some()
}

/// True when an env var is set to a non-empty value.
///
/// GitHub Actions exports undefined secrets as empty strings (`${{ secrets.X }}`
/// resolves to `""` when `X` is not configured). Plain `env::var(k).is_ok()`
/// would treat that as "present", allowing live tests to run with blank
/// credentials. This helper rejects both unset and empty-string values.
fn env_is_present(key: &str) -> bool {
    env::var(key).map(|v| !v.is_empty()).unwrap_or(false)
}

/// True only when running inside a GitHub Actions workflow.
///
/// `GITHUB_ACTIONS` is the authoritative signal — it is set automatically
/// by every GitHub Actions runner. The generic `CI` env var is deliberately
/// **not** checked because many non-CI tools (Cursor IDE, iTerm, etc.) set
/// it, which causes tests to panic instead of gracefully skipping.
fn in_github_actions() -> bool {
    env::var("GITHUB_ACTIONS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

/// Guard a test based on its metadata.
///
/// Returns true if the test should run, false if it should be skipped.
///
/// In GitHub Actions with the default cost budget, exceeding the budget or
/// missing secrets causes a panic (to catch CI misconfigurations). When the
/// cost limit is explicitly set via `GUNBC_TEST_MAX_COST`, tests that exceed
/// the budget are silently skipped — the caller made a deliberate choice.
pub fn guard(meta: TestMeta<'_>) -> bool {
    let max_cost = max_cost_from_env();
    let explicit = cost_limit_is_explicit();

    if meta.cost > max_cost {
        if in_github_actions() && !explicit {
            panic!(
                "skipping {}: cost {} exceeds max {} (set GUNBC_TEST_MAX_COST=...)",
                meta.name,
                meta.cost.as_str(),
                max_cost.as_str()
            );
        }
        eprintln!(
            "[guard] skipping {}: cost {} exceeds max {}",
            meta.name,
            meta.cost.as_str(),
            max_cost.as_str()
        );
        return false;
    }

    if !meta.secrets.is_empty() {
        let missing: Vec<&str> = meta
            .secrets
            .iter()
            .copied()
            .filter(|k| !env_is_present(k))
            .collect();
        if !missing.is_empty() {
            if in_github_actions() && !explicit {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    meta.name,
                    missing.join(", ")
                );
            }
            eprintln!(
                "[guard] skipping {}: missing secrets [{}]",
                meta.name,
                missing.join(", ")
            );
            return false;
        }
    }

    if !meta.requires.is_empty() {
        // We don't probe the environment; this is informational only.
        // If a test fails due to missing deps, users can adjust their setup.
        let _ = meta.requires;
    }

    true
}

/// Convenience helper without building TestMeta manually.
pub fn guard_test(
    name: &str,
    class: TestClass,
    cost: FermiCost,
    requires: &[&str],
    secrets: &[&str],
) -> bool {
    guard(TestMeta {
        name,
        class,
        cost,
        requires,
        secrets,
    })
}

/// Guard helper for env requirements that include "any-of" groups.
///
/// `required` are checked directly. Each group in `required_any_of` requires at
/// least one env var to be present.
///
/// Like [`guard`], panics in CI are suppressed when `GUNBC_TEST_MAX_COST` is
/// explicitly set.
pub fn guard_test_with_env(
    name: &str,
    _class: TestClass,
    cost: FermiCost,
    requires: &[&str],
    required: &[&str],
    required_any_of: &[&[&str]],
) -> bool {
    let max_cost = max_cost_from_env();
    let explicit = cost_limit_is_explicit();

    if cost > max_cost {
        if in_github_actions() && !explicit {
            panic!(
                "skipping {}: cost {} exceeds max {} (set GUNBC_TEST_MAX_COST=...)",
                name,
                cost.as_str(),
                max_cost.as_str()
            );
        }
        eprintln!(
            "[guard] skipping {}: cost {} exceeds max {}",
            name,
            cost.as_str(),
            max_cost.as_str()
        );
        return false;
    }

    if !required.is_empty() {
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|k| !env_is_present(k))
            .collect();
        if !missing.is_empty() {
            if in_github_actions() && !explicit {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    name,
                    missing.join(", ")
                );
            }
            eprintln!(
                "[guard] skipping {}: missing secrets [{}]",
                name,
                missing.join(", ")
            );
            return false;
        }
    }

    if !required_any_of.is_empty() {
        let mut missing_groups: Vec<String> = Vec::new();
        for group in required_any_of {
            let present = group.iter().any(|k| env_is_present(k));
            if !present {
                missing_groups.push(group.join(" | "));
            }
        }
        if !missing_groups.is_empty() {
            if in_github_actions() && !explicit {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    name,
                    missing_groups.join(", ")
                );
            }
            eprintln!(
                "[guard] skipping {}: missing secrets [{}]",
                name,
                missing_groups.join(", ")
            );
            return false;
        }
    }

    if !requires.is_empty() {
        let _ = requires;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env-var tests must run serially to avoid races on global state.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper: run `f` with the given env overrides, restoring afterward.
    fn with_env(overrides: &[(&str, Option<&str>)], f: impl FnOnce()) {
        // Accept poisoned lock: prior #[should_panic] tests may have poisoned it.
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved: Vec<(&str, Option<String>)> = overrides
            .iter()
            .map(|(k, _)| (*k, env::var(k).ok()))
            .collect();
        for (k, v) in overrides {
            match v {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        for (k, v) in &saved {
            match v {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }
        // Release lock before re-panicking to avoid poisoning the mutex.
        drop(lock);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_cost_limit_is_explicit_when_set() {
        with_env(&[("GUNBC_TEST_MAX_COST", Some("S"))], || {
            assert!(cost_limit_is_explicit());
        });
    }

    #[test]
    fn test_cost_limit_is_not_explicit_when_unset() {
        with_env(&[("GUNBC_TEST_MAX_COST", None)], || {
            assert!(!cost_limit_is_explicit());
        });
    }

    #[test]
    fn test_cost_limit_is_not_explicit_for_invalid_value() {
        with_env(&[("GUNBC_TEST_MAX_COST", Some("bogus"))], || {
            assert!(!cost_limit_is_explicit());
        });
    }

    #[test]
    fn test_guard_skips_silently_with_explicit_cost_limit_in_ci() {
        // Simulates the preflight scenario: GITHUB_ACTIONS=true + explicit cost limit.
        // The guard should return false (skip), NOT panic.
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", Some("S")),
            ],
            || {
                let result = guard(TestMeta {
                    name: "expensive_test",
                    class: TestClass::Integration,
                    cost: FermiCost::M,
                    requires: &[],
                    secrets: &[],
                });
                assert!(!result, "test should be skipped, not run");
            },
        );
    }

    #[test]
    #[should_panic(expected = "missing secrets")]
    fn test_guard_panics_on_missing_secrets_in_ci_without_explicit_limit() {
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", None),
                // Ensure the secret is missing
                ("NONEXISTENT_SECRET_FOR_TEST", None),
            ],
            || {
                guard(TestMeta {
                    name: "secret_test",
                    class: TestClass::Integration,
                    cost: FermiCost::XS, // within budget
                    requires: &[],
                    secrets: &["NONEXISTENT_SECRET_FOR_TEST"],
                });
            },
        );
    }

    #[test]
    fn test_guard_skips_missing_secrets_with_explicit_cost_limit() {
        // With explicit cost limit, missing secrets should skip, not panic.
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", Some("XL")),
                ("NONEXISTENT_SECRET_FOR_TEST", None),
            ],
            || {
                let result = guard(TestMeta {
                    name: "secret_test",
                    class: TestClass::Integration,
                    cost: FermiCost::XS,
                    requires: &[],
                    secrets: &["NONEXISTENT_SECRET_FOR_TEST"],
                });
                assert!(!result, "test should be skipped, not run");
            },
        );
    }

    #[test]
    fn test_guard_test_with_env_skips_with_explicit_limit() {
        // Simulates the exact CI failure scenario: live flow test with required
        // secrets, running in GitHub Actions with an explicit cost limit.
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", Some("S")),
            ],
            || {
                let result = guard_test_with_env(
                    "test_live_flow",
                    TestClass::Integration,
                    FermiCost::M,
                    &["fs", "shell"],
                    &["GCP_WIF_PROVIDER", "GCP_SECRETS_PROJECT"],
                    &[&["GCP_SECRETS_SA", "GCP_SECRETS_IMPERSONATE_SA"]],
                );
                assert!(!result, "live test should be skipped with explicit cost limit");
            },
        );
    }

    #[test]
    #[should_panic(expected = "missing secrets")]
    fn test_guard_test_with_env_panics_without_explicit_limit() {
        // Without explicit limit, missing secrets should panic in CI.
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", None),
            ],
            || {
                guard_test_with_env(
                    "test_live_flow",
                    TestClass::Integration,
                    FermiCost::XS, // within default XL budget
                    &[],
                    &["DEFINITELY_MISSING_SECRET"],
                    &[],
                );
            },
        );
    }

    #[test]
    fn test_env_is_present_rejects_empty_string() {
        // GitHub Actions exports undefined secrets as "" — guard must treat that as missing.
        with_env(&[("EMPTY_SECRET_TEST", Some(""))], || {
            assert!(
                !env_is_present("EMPTY_SECRET_TEST"),
                "empty string should be treated as absent"
            );
        });
    }

    #[test]
    fn test_env_is_present_accepts_non_empty() {
        with_env(&[("PRESENT_SECRET_TEST", Some("value"))], || {
            assert!(env_is_present("PRESENT_SECRET_TEST"));
        });
    }

    #[test]
    fn test_env_is_present_rejects_unset() {
        with_env(&[("MISSING_SECRET_TEST", None)], || {
            assert!(!env_is_present("MISSING_SECRET_TEST"));
        });
    }

    #[test]
    #[should_panic(expected = "missing secrets")]
    fn test_guard_panics_on_empty_string_secret_in_ci() {
        // Empty-string secrets (from undefined GitHub Actions secrets) must be
        // detected as missing, not silently treated as present.
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", None),
                ("EMPTY_GCP_SECRET_TEST", Some("")),
            ],
            || {
                guard(TestMeta {
                    name: "empty_secret_test",
                    class: TestClass::Integration,
                    cost: FermiCost::XS,
                    requires: &[],
                    secrets: &["EMPTY_GCP_SECRET_TEST"],
                });
            },
        );
    }

    #[test]
    #[should_panic(expected = "missing secrets")]
    fn test_guard_test_with_env_panics_on_empty_string_required() {
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", None),
                ("EMPTY_REQUIRED_TEST", Some("")),
            ],
            || {
                guard_test_with_env(
                    "empty_required_test",
                    TestClass::Integration,
                    FermiCost::XS,
                    &[],
                    &["EMPTY_REQUIRED_TEST"],
                    &[],
                );
            },
        );
    }

    #[test]
    #[should_panic(expected = "missing secrets")]
    fn test_guard_test_with_env_panics_on_empty_string_any_of() {
        with_env(
            &[
                ("GITHUB_ACTIONS", Some("true")),
                ("GUNBC_TEST_MAX_COST", None),
                ("EMPTY_GROUP_A", Some("")),
                ("EMPTY_GROUP_B", Some("")),
            ],
            || {
                guard_test_with_env(
                    "empty_any_of_test",
                    TestClass::Integration,
                    FermiCost::XS,
                    &[],
                    &[],
                    &[&["EMPTY_GROUP_A", "EMPTY_GROUP_B"]],
                );
            },
        );
    }

    #[test]
    fn test_guard_runs_within_budget() {
        with_env(
            &[
                ("GITHUB_ACTIONS", None),
                ("GUNBC_TEST_MAX_COST", Some("M")),
            ],
            || {
                let result = guard(TestMeta {
                    name: "cheap_test",
                    class: TestClass::Hermetic,
                    cost: FermiCost::S,
                    requires: &[],
                    secrets: &[],
                });
                assert!(result, "test within budget should run");
            },
        );
    }
}
