//! R3 gate **#102** (`slow_test_exemptions_dissolved`) — project per-test wall-clock
//! warn policy from substrate (`dsl/gunbc/test_node_wall_clock_ratchet.dag`) instead
//! of a checked-in JSONL side manifest.

use crate::dag::{FieldValue, LiteralBits, ValueBody};
use crate::Dag;

/// Repo-relative path to the modeled warn-token table (single edit surface).
pub const RATCHET_DAG_REL_PATH: &str = "dsl/gunbc/test_node_wall_clock_ratchet.dag";

/// One JSON object per line: `{"test":"<libtest token>","policy":"warn"}` — the
/// shape `scripts/check-test-timeout.sh` / `jq` already consume.
pub fn emit_warn_policy_jsonl_lines(dag: &Dag) -> Result<Vec<String>, String> {
    let decl = dag
        .declaration_by_name("wall_clock_warn_libtest_tokens")
        .ok_or_else(|| {
            "missing `wall_clock_warn_libtest_tokens` data row in ratchet .dag".to_string()
        })?;
    let body = decl
        .value_body
        .as_ref()
        .ok_or_else(|| "wall_clock_warn_libtest_tokens: no value_body".to_string())?;
    let ValueBody::List(rows) = body else {
        return Err(format!(
            "wall_clock_warn_libtest_tokens: expected ValueBody::List, got {body:?}"
        ));
    };
    let mut out = Vec::with_capacity(rows.len());
    for (idx, row) in rows.iter().enumerate() {
        let fields = match row {
            FieldValue::Record(f) => f.as_slice(),
            other => {
                return Err(format!(
                    "row {idx}: expected record `WallClockWarnLibtestToken`, got {other:?}"
                ));
            }
        };
        let test_val = fields
            .iter()
            .find(|(l, _)| l == "test")
            .map(|(_, v)| v)
            .ok_or_else(|| format!("row {idx}: missing `test` field"))?;
        let test = match test_val {
            FieldValue::Literal(LiteralBits::String(s)) => s.as_str(),
            other => {
                return Err(format!(
                    "row {idx}: `test` must be a String literal, got {other:?}"
                ));
            }
        };
        out.push(format!(
            "{{\"test\":{},\"policy\":\"warn\"}}",
            json_escape_string(test)
        ));
    }
    Ok(out)
}

fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write;
                let _ = write!(&mut out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;
    use serde_json::Value;
    use std::path::PathBuf;

    fn load_ratchet_dag() -> Dag {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("../../../dsl/gunbc/test_node_wall_clock_ratchet.dag");
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("read {}: {e}", path.display());
        });
        compile_to_dag(&source, path.to_string_lossy().as_ref()).unwrap_or_else(|err| {
            panic!("compile {}: {err:?}", path.display());
        })
    }

    #[test]
    fn ratchet_dag_warn_manifest_lines_are_parseable_warn_policy_objects() {
        let dag = load_ratchet_dag();
        let lines = emit_warn_policy_jsonl_lines(&dag).expect("emit");
        for (idx, line) in lines.iter().enumerate() {
            let v: Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("line {idx} must be JSON: {e}; got {line:?}"));
            let obj = v.as_object().unwrap_or_else(|| {
                panic!("line {idx} must be a JSON object; got {v:?}");
            });
            assert!(
                obj.get("test").and_then(Value::as_str).is_some(),
                "line {idx} must have string `test`"
            );
            assert_eq!(
                obj.get("policy").and_then(Value::as_str),
                Some("warn"),
                "line {idx} must have policy=warn"
            );
        }
    }
}
