//! One-shot diagnostic: find TestClaim modules where selection machinery fails.
#![allow(clippy::disallowed_macros)]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::Command;

use v1_compiler::cli_run::{
    build_multi_entry_index, make_eval_context, resolve_entry_with_index, workspace_root,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn pr_changed_files() -> Vec<String> {
    let ws = workspace_root();
    let out = Command::new("git")
        .args(["diff", "--name-only", "origin/main...HEAD"])
        .current_dir(&ws)
        .output()
        .expect("git diff");
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

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

fn scan_entry(
    rel: &str,
    index: &v1_compiler::cli_run::MultiEntryIndex,
    empty_frontier: &Value,
) -> Vec<String> {
    let ws = workspace_root();
    let abs = ws.join(rel);
    if !abs.is_file() {
        return vec![];
    }
    let entry = abs.to_string_lossy().into_owned();
    let Ok((graph, source_indices)) = resolve_entry_with_index(index, &entry) else {
        return vec![format!("{rel}: resolve failed")];
    };
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
    let initializer_values = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::eval_data_initializer_values(&ctx)
    }) {
        Ok(v) => v,
        Err(e) => return vec![format!("{rel}: eval_data_initializer_values: {e}")],
    };
    let mut failures = Vec::new();
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
            (Some("frontier".to_string()), empty_frontier.clone()),
        ];
        if let Err(e) = v1_interpreter::run_in_context_with_args(
            &ctx,
            "test_claim_evaluation_touches_rerun_frontier",
            &args,
            false,
        ) {
            failures.push(format!(
                "{rel} claim `{}`: {e}",
                claim_label(&val, &ctx)
            ));
        }
    }
    failures
}

#[test]
fn pr_changed_test_claim_modules_pass_selection_machinery() {
    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir");
    let roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dag").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);
    let empty_frontier = Value::List(std::rc::Rc::new(im_rc::Vector::new()));
    let changed = pr_changed_files();
    let claim_files: std::collections::HashSet<String> =
        dag_files_with_test_claims().into_iter().collect();

    let mut targets: Vec<String> = claim_files
        .iter()
        .filter(|f| changed.iter().any(|c| *f == c || f.ends_with(c.as_str())))
        .cloned()
        .collect();
    targets.sort();

    // Also scan cost lens claims pulled in by C1 imports.
    for extra in [
        "src/v2/lens/cost/loop_linear.dag",
        "src/v2/lens/cost/map_linear.dag",
        "src/v2/lens/cost/disj_max.dag",
        "src/v2/lens/cost/nested_product.dag",
        "src/v2/lens/cost/loop_unknown.dag",
        "src/v2/lens/cost/loop_illegal_named.dag",
        "src/v2/lens/cost/atom_zero_test.dag",
        "src/v2/lens/cost/p9_llvm_instruction_cost_registry_owner.dag",
        "src/v2/test/claim/workflow/provenance_fail_closed_contract_test.dag",
    ] {
        if !targets.contains(&extra.to_string()) {
            targets.push(extra.to_string());
        }
    }
    targets.sort();
    targets.dedup();

    let mut failures = Vec::new();
    for rel in &targets {
        eprintln!("scanning {rel}");
        let rel = rel.clone();
        let index = &index;
        let empty_frontier = &empty_frontier;
        match catch_unwind(AssertUnwindSafe(|| scan_entry(&rel, index, empty_frontier))) {
            Ok(mut f) => failures.append(&mut f),
            Err(_) => failures.push(format!("{rel}: panic during scan")),
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
