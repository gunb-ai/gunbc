//! Freshness policy stubs.
//!
//! The monolithic freshness system (lint_upsert) has been removed. Each DSL
//! tool is self-checking via `content_upsert` — the atomic read-compare-write
//! pattern. The DSL compiler knows each tool's import closure, and `ci.dag`
//! models the correct pipeline ordering.
//!
//! These stubs preserve the public API so generated binaries compile without
//! changes. They return "always fresh" — no freshness gate runs.
//!
//! See `docs/design/freshness-overhaul.md` for the full rationale.

use gunbc_exec::FreshnessStep;

/// Always returns `None` — no freshness steps are injected.
///
/// Each tool handles its own freshness via `content_upsert`. The monolithic
/// 645-file hash and 7-step sequential chain have been removed.
pub fn check_and_plan_freshness() -> Option<Vec<FreshnessStep>> {
    None
}

/// Always returns `None` — same as `check_and_plan_freshness`.
pub fn check_and_plan_generation_freshness() -> Option<Vec<FreshnessStep>> {
    None
}

/// No-op — the monolithic manifest entry has been removed.
pub fn update_freshness_manifest() -> Result<(), String> {
    Ok(())
}
