use crate::helpers::workspace_root;
use std::fs;

use v1_compiler::cli_run::{
    self, build_multi_entry_index, make_eval_context, resolve_entry_graph,
    resolve_entry_with_index, run_claim, ClaimOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn outcome_tag(o: &ClaimOutcome) -> String {
    match o {
        ClaimOutcome::Pass => "PASS".to_string(),
        ClaimOutcome::Fail => "FAIL".to_string(),
        ClaimOutcome::NotBool { got } => format!("NOTBOOL({got})"),
        ClaimOutcome::RuntimeError { message } => format!("RUNTIMEERR({message})"),
    }
}

fn cold_oracle(roots: &[String], entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("cold resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

fn cached(index: &cli_run::MultiEntryIndex, entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_with_index(index, entry).expect("cached resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

#[test]
fn typed_module_cache_matches_cold_oracle_in_every_order() {
    // Workspace-relative fixture root (target/ is gitignored + ephemeral):
    // build_module_path_index fails closed on paths outside the workspace, so
    // std::env::temp_dir() fixtures cannot resolve (same precedent as
    // union_resolve_receipts_test).
    let dir = workspace_root()
        .join("target")
        .join(format!("gunbc-typed-cache-eq-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");

    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_a_true() -> Bool { (unbox(boxed(val())) + 0) == 10 }\n\
        fn witness_a_false() -> Bool { val() == 999 }\n";
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        fn witness_b_true() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";
    let entry_c = "module test.c\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_c_true() -> Bool { unbox(boxed(val() + 5)) == 15 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("shared2.dag", shared2),
        ("extra.dag", extra),
        ("entry_a.dag", entry_a),
        ("entry_b.dag", entry_b),
        ("entry_c.dag", entry_c),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }

    let roots = vec![dir.to_string_lossy().into_owned()];
    let a = dir.join("entry_a.dag").to_string_lossy().into_owned();
    let b = dir.join("entry_b.dag").to_string_lossy().into_owned();
    let c = dir.join("entry_c.dag").to_string_lossy().into_owned();

    let witnesses = [
        (&a, "witness_a_true", "PASS"),
        (&a, "witness_a_false", "FAIL"),
        (&b, "witness_b_true", "PASS"),
        (&c, "witness_c_true", "PASS"),
    ];

    for (entry, f, expected) in witnesses {
        let cold = cold_oracle(&roots, entry, f);
        assert_eq!(cold, expected, "cold oracle unexpected for {f}");
    }

    let orders: [&[&str]; 3] = [&[&a, &b, &c], &[&c, &b, &a], &[&b, &a, &c]];
    for order in orders {
        let index = build_multi_entry_index(&roots);
        for entry in order {
            let _ = resolve_entry_with_index(&index, entry).expect("warm resolve");
        }
        for (entry, f, expected) in witnesses {
            let got = cached(&index, entry, f);
            assert_eq!(
                got, expected,
                "cached verdict for {f} diverged from cold oracle in order {order:?}"
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}
