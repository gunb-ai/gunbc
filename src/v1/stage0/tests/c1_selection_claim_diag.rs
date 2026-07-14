//! One-shot diagnostic: find TestClaim modules where selection machinery fails.
#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::Command;

use v1_compiler::cli_run::{
    build_multi_entry_index, entry_touches_rerun_frontier, list_value_from_vec,
    make_eval_context, resolve_entry_with_index, workspace_root,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn dag_files_with_test_claims() -> Vec<String> {
    let ws = workspace_root();
    let out = Command::new("rg")
        .args(["-l", "TestClaim =", "--glob", "*.dag"])
        .current_dir(&ws)
        .output()
        .expect("rg");
    assert!(out.status.success(), "rg failed");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn claim_label(val: &Value, ctx: &v1_compiler::v1_interpreter::InterpContext) -> String {
    match val {
        Value::Variant { fields, .. } => ctx
            .field(fields, "label")
            .and_then(|v| match v {
                Value::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "<no label>".to_string()),
        _ => format!("{:?}", val),
    }
}

#[test]
fn all_test_claim_modules_pass_selection_machinery() {
    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir");
    let roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dag").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);
    let empty_frontier = list_value_from_vec(vec![]);
    let mut failures: Vec<String> = Vec::new();

    for rel in dag_files_with_test_claims() {
        let abs = ws.join(&rel);
        if !abs.is_file() {
            continue;
        }
        let entry = abs.to_string_lossy().into_owned();
        let Ok((graph, source_indices)) = resolve_entry_with_index(&index, &entry) else {
            continue;
        };
        let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        if let Err(e) = entry_touches_rerun_frontier(&ctx, &empty_frontier) {
            failures.push(format!("{rel}: {e}"));
            continue;
        }
        // Also locate which claim breaks test_claim_evaluation_nodes directly.
        let initializer_values = v1_interpreter::with_active_context(&ctx, || {
            v1_interpreter::eval_data_initializer_values(&ctx)
        })
        .expect("eval_data_initializer_values");
        for val in initializer_values {
            let Value::Variant { variant_name, .. } = &val else {
                continue;
            };
            if !matches!(
                ctx.resolve(*variant_name).as_str(),
                "EqualsClaim"
                    | "CompilesClaim"
                    | "DiagnosticClaim"
                    | "StructuralEqualsClaim"
                    | "RoundTripClaim"
                    | "BoolWitnessClaim"
            ) {
                continue;
            }
            let args = [
                (Some("c".to_string()), val.clone()),
                (
                    Some("frontier".to_string()),
                    empty_frontier.clone(),
                ),
            ];
            if let Err(e) =
                v1_interpreter::run_in_context_with_args(&ctx, "test_claim_evaluation_touches_rerun_frontier", &args, false)
            {
                failures.push(format!(
                    "{rel} claim `{}`: test_claim_evaluation_touches_rerun_frontier: {e}",
                    claim_label(&val, &ctx)
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "selection machinery failures ({}):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
