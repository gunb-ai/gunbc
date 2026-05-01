//! P0 prerequisite pin: stable top-level JSON keys in `target/self_host/receipt.json`.
//!
//! Authority (workspace-root paths as code — not file-relative rustdoc URLs):
//! `docs/briefs/r3-pb-t-fixedpoint-worker.md` §P0 readiness checklist (DB-8 mechanical ratchet);
//! `docs/db-history/db-8.md`; `docs/design-fixed-point-ratchet.md`.
//! `self_host_fixed_point` consumes these identifiers so renames are deliberate (trend readers / DB-8).

/// Pipeline snapshot fixed-point on [`crate::default_fixed_point_source`] (always `ok` when the binary runs past that stage).
pub const K_PIPELINE_FIXED_POINT_DEFAULT_SOURCE: &str = "pipeline_fixed_point_default_source";

/// `dsl/gunbc/compiler.dag` parse outcome under v3 (`ok` or encoded error string).
pub const K_COMPILER_DAG_V3_PARSE: &str = "compiler_dag_v3_parse";

/// Overall receipt status (`completed` or `failed_self_host_slice` today).
pub const K_STATUS: &str = "status";

/// Keys emitted on every path (parse failure still includes pipeline + parse + status).
pub const ALWAYS_EMITTED_TOP_LEVEL_KEYS: &[&str] = &[
    K_PIPELINE_FIXED_POINT_DEFAULT_SOURCE,
    K_COMPILER_DAG_V3_PARSE,
    K_STATUS,
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn always_emitted_keys_are_unique_nonempty() {
        let mut seen = HashSet::new();
        for key in super::ALWAYS_EMITTED_TOP_LEVEL_KEYS {
            assert!(!key.is_empty(), "empty key");
            assert!(seen.insert(*key), "duplicate key {key}");
        }
    }
}
