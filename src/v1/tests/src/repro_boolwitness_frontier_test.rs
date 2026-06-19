//! TEMPORARY repro for the boolwitness frontier fold error.
use std::path::PathBuf;

use v1_compiler::cli_run::{build_multi_entry_index, make_eval_context, resolve_entry_with_index};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn ws() -> PathBuf {
    crate::helpers::workspace_root()
}

#[test]
fn repro_boolwitness_passed_to_test_claim_evaluation_nodes() {
    let w = ws();
    let roots = vec![
        w.join("src/v2").to_string_lossy().into_owned(),
        w.join("dsl").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);
    let entry = w
        .join("src/v2/compiler/manual/timeseries_passive_map_fold_anchor_test.dag")
        .to_string_lossy()
        .into_owned();
    let (graph, source_indices) =
        resolve_entry_with_index(&index, &entry).expect("resolve timeseries entry");
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);

    // Eval a `: TestClaim = BoolWitnessClaim {...}` decl.
    let val = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::eval_data_item_value(&ctx, "claim_timeseries_resistor_map")
    })
    .expect("eval data")
    .expect("decl exists");

    match &val {
        Value::Variant { variant_name, .. } => {
            eprintln!("REPRO variant_name = {}", ctx.resolve(*variant_name));
        }
        other => eprintln!("REPRO not a variant: {}", ctx.format_value(other)),
    }

    // Now mimic call_test_claim_fn_nodes: pass it as `c` to test_claim_evaluation_nodes.
    let args = [(Some("c".to_string()), val.clone())];
    let result =
        v1_interpreter::run_in_context_with_args(&ctx, "test_claim_evaluation_nodes", &args, false);
    match result {
        Ok(v) => eprintln!("REPRO OK: {}", ctx.format_value(&v)),
        Err(e) => eprintln!("REPRO ERR: {}", e),
    }

    // Now a REAL CompilesClaim through the same fn (should fold to a 2-node list).
    let compiles = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::eval_data_item_value(&ctx, "claim_timeseries_passive_map_fold_compiles")
    })
    .expect("eval compiles data")
    .expect("compiles decl exists");
    let args2 = [(Some("c".to_string()), compiles.clone())];
    match v1_interpreter::run_in_context_with_args(&ctx, "test_claim_evaluation_nodes", &args2, false)
    {
        Ok(v) => eprintln!("REPRO COMPILES-OK: {}", ctx.format_value(&v)),
        Err(e) => eprintln!("REPRO COMPILES-ERR: {}", e),
    }

    // Now mimic entry_claims_touch_frontier: eval_data_initializer_values is NOT wrapped in
    // with_active_ctx. Does it fold? (No fold inside, but the touches fn calls it.)
    let init_vals = v1_interpreter::eval_data_initializer_values(&ctx);
    match init_vals {
        Ok(vs) => eprintln!("REPRO INIT-VALS count = {}", vs.len()),
        Err(e) => eprintln!("REPRO INIT-VALS-ERR: {}", e),
    }
}
