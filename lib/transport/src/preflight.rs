//! Preflight module — removed.
//!
//! The monolithic `LintResource` and `run_lint_upsert` have been deleted.
//! Each DSL tool is self-checking via `content_upsert`. The hardcoded glob
//! list (18 patterns, ~645 files) duplicated `dsl/config/resources.dag`.
//! The `PREFLIGHT_SKIP_BINARIES` recursion guard was a manual workaround
//! for a problem that disappears with per-tool freshness.
//!
//! See `docs/design/freshness-overhaul.md` for the full rationale.
