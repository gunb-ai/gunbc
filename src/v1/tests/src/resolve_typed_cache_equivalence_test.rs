//! Falsifier + permanent regression guard for the resolve-cost lever (PR1): the
//! per-module typed-module cache on `MultiEntryIndex` must make every witness's
//! resolve verdict BYTE-IDENTICAL to the no-cache cold single-entry resolve (the
//! production oracle), in EVERY entry order.
//!
//! This is the witness for the born-marked soundness property in cli_run.rs
//! (`seed_kernel_intern_names`): a module's typed result is content-pure — a pure
//! function of (its content + its imports' identities) — ONLY IF the type-time
//! kernel intern ids are content-stable. The cache exposed that without the
//! kernel-name pre-seed they are table-SIZE dependent, so a module cached for an
//! early entry and reused by a later entry mismatched its kernel ids and types
//! collapsed to the `Json` fallback — an order-dependent verdict flip. The seed
//! restores content-stability; this test fails if that ever regresses.
//!
//! ORACLE = `resolve_entry_graph` (single entry → `resolved_graph_from_sources`
//! → `reconcile`, no typed cache). CACHED = `resolve_entry_with_index` over one
//! shared `MultiEntryIndex`. The test resolves the entries in MULTIPLE orders
//! through independent indices and asserts cached == cold-oracle for each, since
//! order-agnostic correctness (not cross-order agreement) is the property.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Cold oracle: resolve `entry` ALONE (no typed cache) and classify `function`.
fn cold_oracle(roots: &[String], entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("cold resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

/// Cached: resolve `entry` through the shared index and classify `function`.
fn cached(index: &cli_run::MultiEntryIndex, entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_with_index(index, entry).expect("cached resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

#[test]
fn typed_module_cache_matches_cold_oracle_in_every_order() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "gunbc-typed-cache-eq-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("temp dir");

    // A shared module that USES a kernel type (Optional via a record field) — the
    // kernel-id path that the seed stabilizes — reused across entries.
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    // Two distinct modules declaring the SAME unqualified item name `val` with
    // DIFFERENT values — a name-keyed cache must not alias them.
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    // entry_a: SMALL closure (common + shared1).
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_a_true() -> Bool { (unbox(boxed(val())) + 0) == 10 }\n\
        fn witness_a_false() -> Bool { val() == 999 }\n";
    // entry_b: LARGER closure (common + shared2 + extra) so its table size differs
    // from entry_a's — the condition under which unstable kernel ids would bite.
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        fn witness_b_true() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";
    // entry_c: re-uses common + shared1, distinct witness.
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

    // (entry, function, expected verdict) — expected also pinned so a uniform
    // wrong answer (e.g. all-FAIL) cannot pass by mere cross-order agreement.
    let witnesses = [
        (&a, "witness_a_true", "PASS"),
        (&a, "witness_a_false", "FAIL"),
        (&b, "witness_b_true", "PASS"),
        (&c, "witness_c_true", "PASS"),
    ];

    // Cold oracle per witness (ground truth).
    for (entry, f, expected) in witnesses {
        let cold = cold_oracle(&roots, entry, f);
        assert_eq!(cold, expected, "cold oracle unexpected for {f}");
    }

    // Resolve through the shared index in several orders; each must equal the cold
    // oracle. Each order uses a FRESH index so the prefix (and thus table growth)
    // differs — the exact condition that broke before the kernel-id seed.
    let orders: [&[&str]; 3] = [&[&a, &b, &c], &[&c, &b, &a], &[&b, &a, &c]];
    for order in orders {
        let index = build_multi_entry_index(&roots);
        // Warm the index by resolving the entries in this order first.
        for entry in order {
            let _ = resolve_entry_with_index(&index, entry).expect("warm resolve");
        }
        // Then check every witness against its cold oracle on the warm index.
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
