use crate::module_path_index::parsed_dag_file::parse_dag_file;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_interpreter::{self, sorted_fields, str_value, ExecutionMode, InterpContext, Value};
use crate::v1_std_core::{authored_name_at, expr_call_func_at, expr_var_name_at, ExprData, Node};
use im::HashMap;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const TEST_MODULE_HYGIENE_ENTRY: &str = "dag/gunbc/test_module_hygiene.dag";

fn hygiene_entry_file() -> String {
    let path = Path::new(TEST_MODULE_HYGIENE_ENTRY);
    if path.is_file() {
        return path.to_string_lossy().into_owned();
    }
    let ws_path = super::workspace_root().join(path);
    if ws_path.is_file() {
        return ws_path.to_string_lossy().into_owned();
    }
    TEST_MODULE_HYGIENE_ENTRY.to_string()
}

pub(crate) fn is_file_grain_function(function: &str) -> bool {
    function.is_empty()
}

fn test_decl_names_from_content(content: &str) -> Result<Vec<String>, String> {
    let roots = super::default_source_roots();
    let ctx = resolve_hygiene_ctx(&roots)?;
    let args = [(Some("content".to_string()), str_value(content.to_string()))];
    let result =
        v1_interpreter::run_in_context_with_args(&ctx, "enumerate_entry_test_names", &args, false)
            .map_err(|e| format!("enumerate_entry_test_names: {e}"))?;
    let Value::List(items) = result else {
        return Err(format!(
            "enumerate_entry_test_names returned `{}`, expected List",
            ctx.format_value(&result)
        ));
    };
    items
        .iter()
        .map(|item| match item {
            Value::Str(s) => Ok(s.to_string()),
            other => Err(format!(
                "enumerate_entry_test_names element `{}` is not String",
                ctx.format_value(other)
            )),
        })
        .collect()
}

fn repo_relative(path: &Path, ws: &Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn resolve_hygiene_ctx(source_roots: &[String]) -> Result<InterpContext, String> {
    let entry = hygiene_entry_file();
    let (graph, indices) = super::resolve_entry_graph_shared(source_roots, &entry)
        .map_err(|e| format!("test_module_hygiene resolve: {e}"))?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureReceiptCompanionLookup {
    Declared(String),
    AuthorityRefused { cause: String },
}

pub(crate) fn failure_receipt_companion_from_authority(
    function: &str,
) -> FailureReceiptCompanionLookup {
    let roots = super::default_source_roots();
    let ctx = match resolve_hygiene_ctx(&roots) {
        Ok(ctx) => ctx,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused { cause: detail };
        }
    };
    let args = [(
        Some("function".to_string()),
        str_value(function.to_string()),
    )];
    let result = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "failure_receipt_companion",
        &args,
        false,
    ) {
        Ok(value) => value,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused {
                cause: detail.to_string(),
            };
        }
    };
    // The authority's projection is TOTAL: it returns String, so there is no Absent to
    // decode and no not-declared arm to fall through. A witness whose companion does not
    // exist is distinguished later, by the companion lookup itself returning empty -- not
    // here, by its name.
    match &result {
        Value::Str(companion) => FailureReceiptCompanionLookup::Declared(companion.to_string()),
        other => FailureReceiptCompanionLookup::AuthorityRefused {
            cause: format!("returned {}, expected String", ctx.format_value(other)),
        },
    }
}

pub(crate) fn witness_verdict_diagnostic_companion_from_authority(
    function: &str,
) -> FailureReceiptCompanionLookup {
    let roots = super::default_source_roots();
    let ctx = match resolve_hygiene_ctx(&roots) {
        Ok(ctx) => ctx,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused { cause: detail };
        }
    };
    let args = [(
        Some("function".to_string()),
        str_value(function.to_string()),
    )];
    let result = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "witness_verdict_diagnostic_companion",
        &args,
        false,
    ) {
        Ok(value) => value,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused {
                cause: detail.to_string(),
            };
        }
    };
    // The authority's projection is TOTAL: it returns String, so there is no Absent to
    // decode and no not-declared arm to fall through. A witness whose companion does not
    // exist is distinguished later, by the companion lookup itself returning empty -- not
    // here, by its name.
    match &result {
        Value::Str(companion) => FailureReceiptCompanionLookup::Declared(companion.to_string()),
        other => FailureReceiptCompanionLookup::AuthorityRefused {
            cause: format!("returned {}, expected String", ctx.format_value(other)),
        },
    }
}

pub(crate) fn enumerate_entry_test_fns(entry_path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(entry_path)
        .map_err(|e| format!("file-grain enumerate: read {entry_path}: {e}"))?;
    let names = test_decl_names_from_content(&content)?;
    if names.is_empty() {
        return Err(format!(
            "file-grain enumerate: entry {entry_path} has no test fn / test data declarations"
        ));
    }
    Ok(names)
}

pub(crate) fn expand_explicit_entries(
    explicit_entries: &[(String, String)],
) -> Result<Vec<(String, String)>, String> {
    let roots = super::default_source_roots();
    let ctx = resolve_hygiene_ctx(&roots)?;
    let mut inputs = Vec::new();
    for (entry, function) in explicit_entries {
        let content = if is_file_grain_function(function) {
            std::fs::read_to_string(entry)
                .map_err(|e| format!("file-grain expand read {entry}: {e}"))?
        } else {
            String::new()
        };
        inputs.push(Value::Record {
            type_name: ctx.sym("ExplicitExpandInput"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("entry"), str_value(entry.clone())),
                (ctx.sym("function"), str_value(function.clone())),
                (ctx.sym("content"), str_value(content)),
            ])),
        });
    }
    let args = [(
        Some("inputs".to_string()),
        v1_interpreter::list_value(inputs),
    )];
    let result = v1_interpreter::run_in_context_with_args(
        &ctx,
        "expand_explicit_pairs_or_refuse",
        &args,
        false,
    )
    .map_err(|e| format!("expand_explicit_pairs_or_refuse: {e}"))?;
    match &result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "ExpandExplicitPairsOutcome")
            && ctx.sym_eq(*variant_name, "Refused") =>
        {
            match ctx.field(fields, "reason") {
                Some(Value::Str(reason)) => Err(reason.to_string()),
                _ => Err("ExpandExplicitPairsOutcome.Refused missing reason".to_string()),
            }
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "ExpandExplicitPairsOutcome")
            && ctx.sym_eq(*variant_name, "Expanded") =>
        {
            let Value::List(items) = ctx
                .field(fields, "pairs")
                .ok_or_else(|| "ExpandExplicitPairsOutcome.Expanded missing pairs".to_string())?
            else {
                return Err("ExpandExplicitPairsOutcome.Expanded pairs is not List".to_string());
            };
            let mut out = Vec::new();
            for item in items.iter() {
                let Value::Record { fields, .. } = item else {
                    return Err(
                        "expand_explicit_pairs_or_refuse element is not ExplicitEntryPair"
                            .to_string(),
                    );
                };
                let entry = match ctx.field(fields, "entry") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => return Err("ExplicitEntryPair missing entry".to_string()),
                };
                let function = match ctx.field(fields, "function") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => return Err("ExplicitEntryPair missing function".to_string()),
                };
                out.push((entry, function));
            }
            Ok(out)
        }
        other => Err(format!(
            "expand_explicit_pairs_or_refuse returned `{}`, expected ExpandExplicitPairsOutcome",
            ctx.format_value(other)
        )),
    }
}

#[cfg(test)]
mod test_module_hygiene_bridge_equivalence_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("gunbc_tmh_bridge_{n}_{id}"))
    }

    #[test]
    fn file_grain_expand_enumerates_test_decls() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("expand_grain_test.dag");
        std::fs::write(
            &file,
            r#"module expand_grain

test fn alpha_holds() -> Bool { true }
test data beta_fixture: Bool = true
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let expanded = expand_explicit_entries(&[(entry, String::new())]).expect("expand");
        let names: BTreeSet<_> = expanded.into_iter().map(|(_, f)| f).collect();
        assert_eq!(
            names,
            BTreeSet::from(["alpha_holds".to_string(), "beta_fixture".to_string()])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_grain_expand_refuses_empty_test_decls() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("empty_grain_test.dag");
        std::fs::write(
            &file,
            r#"module empty_grain

fn plain_only() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = expand_explicit_entries(&[(entry.clone(), String::new())])
            .expect_err("file-grain with no test decls must refuse");
        assert!(
            err.contains("has no test fn / test data declarations"),
            "must refuse empty enumerate: {err}"
        );
        assert!(err.contains(&entry), "must name entry: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failure_receipt_companion_derives_the_suffix_for_both_witness_forms() {
        // Direct coverage for the one convention that survived the orphan-census
        // deletion. It used to be exercised only through the whole-corpus walk in
        // `check_orphan_helpers_or_err`, so without this the fn would have lost its
        // executing consumer along with the machinery it was tested inside.
        assert_eq!(
            failure_receipt_companion_from_authority("w_thing_holds"),
            FailureReceiptCompanionLookup::Declared("w_thing_failure_receipt".to_string()),
        );
        assert_eq!(
            failure_receipt_companion_from_authority("w_thing_passes"),
            FailureReceiptCompanionLookup::Declared("w_thing_failure_receipt".to_string()),
        );
        // The eligibility predicate is deleted: a name carrying neither suffix is no longer
        // "not declared", it simply projects to its own stem. This row is the regression
        // control for that, and it is RED against the pre-change authority.
        assert_eq!(
            failure_receipt_companion_from_authority("ordinary_fn"),
            FailureReceiptCompanionLookup::Declared("ordinary_fn_failure_receipt".to_string()),
        );
    }
}
