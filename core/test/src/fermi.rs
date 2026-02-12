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
            if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
                FermiCost::XL
            } else {
                FermiCost::S
            }
        })
}

fn env_truthy(name: &str) -> bool {
    env::var(name)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

/// Guard a test based on its metadata.
///
/// Returns true if the test should run, false if it should be skipped.
pub fn guard(meta: TestMeta<'_>) -> bool {
    let max_cost = max_cost_from_env();
    if meta.cost > max_cost {
        if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
            panic!(
                "skipping {}: cost {} exceeds max {} (set GUNBC_TEST_MAX_COST=...)",
                meta.name,
                meta.cost.as_str(),
                max_cost.as_str()
            );
        }
        return false;
    }

    if !meta.secrets.is_empty() {
        let missing: Vec<&str> = meta
            .secrets
            .iter()
            .copied()
            .filter(|k| env::var(k).is_err())
            .collect();
        if !missing.is_empty() {
            if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    meta.name,
                    missing.join(", ")
                );
            }
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
pub fn guard_test_with_env(
    name: &str,
    _class: TestClass,
    cost: FermiCost,
    requires: &[&str],
    required: &[&str],
    required_any_of: &[&[&str]],
) -> bool {
    let max_cost = max_cost_from_env();
    if cost > max_cost {
        if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
            panic!(
                "skipping {}: cost {} exceeds max {} (set GUNBC_TEST_MAX_COST=...)",
                name,
                cost.as_str(),
                max_cost.as_str()
            );
        }
        return false;
    }

    if !required.is_empty() {
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|k| env::var(k).is_err())
            .collect();
        if !missing.is_empty() {
            if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    name,
                    missing.join(", ")
                );
            }
            return false;
        }
    }

    if !required_any_of.is_empty() {
        let mut missing_groups: Vec<String> = Vec::new();
        for group in required_any_of {
            let present = group.iter().any(|k| env::var(k).is_ok());
            if !present {
                missing_groups.push(group.join(" | "));
            }
        }
        if !missing_groups.is_empty() {
            if env_truthy("CI") || env_truthy("GITHUB_ACTIONS") {
                panic!(
                    "skipping {}: missing secrets [{}]",
                    name,
                    missing_groups.join(", ")
                );
            }
            return false;
        }
    }

    if !requires.is_empty() {
        let _ = requires;
    }

    true
}
