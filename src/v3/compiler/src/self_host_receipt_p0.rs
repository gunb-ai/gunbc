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

/// Serialized top-level property opener emitted by `self_host_fixed_point` today
/// (`src/v3/compiler/src/bin/self_host_fixed_point.rs`, `run`): two ASCII spaces, `"`, key, `":`
/// — same shape as `format!("  \"{}\":"` / `format!("  \"{}\": {},\n", …` using
/// `K_PIPELINE_FIXED_POINT_DEFAULT_SOURCE`, `K_COMPILER_DAG_V3_PARSE`, and `K_STATUS` (and the
/// parse-error branch). If the emitter changes indentation or switches to `serde_json` pretty-print
/// with different spacing, update this needle in the same change.
fn top_level_property_needle(key: &str) -> String {
    let mut needle = String::with_capacity(key.len() + 8);
    needle.push_str("  \"");
    needle.push_str(key);
    needle.push_str("\":");
    needle
}

fn missing_always_emitted_key_properties(json_body: &str) -> Vec<&'static str> {
    ALWAYS_EMITTED_TOP_LEVEL_KEYS
        .iter()
        .copied()
        .filter(|key| !json_body.contains(&top_level_property_needle(key)))
        .collect()
}

/// True iff [`validate_receipt_json_always_emitted_keys`] would succeed — single matching rule.
pub fn receipt_json_contains_always_emitted_key_properties(json_body: &str) -> bool {
    validate_receipt_json_always_emitted_keys(json_body).is_ok()
}

/// Every [`ALWAYS_EMITTED_TOP_LEVEL_KEYS`] entry must appear as a top-level JSON property using
/// [`top_level_property_needle`]'s shape so the serialized receipt cannot drift from the P0 pin
/// without failing closed before `write_receipt`.
pub fn validate_receipt_json_always_emitted_keys(json_body: &str) -> Result<(), String> {
    let missing = missing_always_emitted_key_properties(json_body);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "receipt.json missing always-emitted P0 keys: {}",
            missing.join(", ")
        ))
    }
}

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

    #[test]
    fn validate_accepts_minimal_receipt_shape() {
        let body = r#"{
  "pipeline_fixed_point_default_source": "ok",
  "compiler_dag_v3_parse": "ok",
  "status": "completed"
}
"#;
        super::validate_receipt_json_always_emitted_keys(body).unwrap();
        assert!(super::receipt_json_contains_always_emitted_key_properties(
            body
        ));
    }

    #[test]
    fn contains_tracks_validate_success_and_failure() {
        let ok = r#"{
  "pipeline_fixed_point_default_source": "ok",
  "compiler_dag_v3_parse": "ok",
  "status": "completed"
}
"#;
        assert_eq!(
            super::receipt_json_contains_always_emitted_key_properties(ok),
            super::validate_receipt_json_always_emitted_keys(ok).is_ok()
        );
        let bad = r#"{
  "compiler_dag_v3_parse": "ok",
  "status": "completed"
}
"#;
        assert_eq!(
            super::receipt_json_contains_always_emitted_key_properties(bad),
            super::validate_receipt_json_always_emitted_keys(bad).is_ok()
        );
    }

    #[test]
    fn validate_rejects_missing_pipeline_key() {
        let body = r#"{
  "compiler_dag_v3_parse": "ok",
  "status": "completed"
}
"#;
        let err = super::validate_receipt_json_always_emitted_keys(body).unwrap_err();
        assert!(err.contains("pipeline_fixed_point_default_source"), "{err}");
    }

    #[test]
    fn validate_rejects_missing_status_key() {
        let body = r#"{
  "pipeline_fixed_point_default_source": "ok",
  "compiler_dag_v3_parse": "x",
}
"#;
        let err = super::validate_receipt_json_always_emitted_keys(body).unwrap_err();
        assert!(err.contains("status"), "{err}");
    }
}
