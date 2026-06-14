//! Falsifier for the resolve-cost lever (PR1): the per-module typed-module cache
//! on `MultiEntryIndex` must make witness verdicts BYTE-IDENTICAL to the
//! uncached single-entry resolve path. A cache that flips a verdict — or that
//! aliases two distinct modules onto one typed result — is a correctness bug,
//! not a perf win, so this exercises the two ways that could happen:
//!
//!   1. A module shared by several entries is type-reconciled once and reused;
//!      every reusing entry's witness must match its uncached verdict.
//!   2. Two DIFFERENT modules that declare the same UNQUALIFIED item name must
//!      NOT collide in the cache (the key is the module name, not the item name).
//!
//! The uncached baseline is `resolve_entry_graph` (single entry →
//! `resolved_graph_from_sources` → `reconcile`, no typed cache); the cached path
//! is `resolve_entry_with_index` over one shared `MultiEntryIndex`.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use v2_compiler::cli_run::{
    self, build_multi_entry_index, make_eval_context, resolve_entry_graph,
    resolve_entry_with_index, run_claim, ClaimOutcome,
};

fn outcome_tag(o: &ClaimOutcome) -> String {
    match o {
        ClaimOutcome::Pass => "PASS".to_string(),
        ClaimOutcome::Fail => "FAIL".to_string(),
        ClaimOutcome::NotBool { got } => format!("NOTBOOL({got})"),
        ClaimOutcome::RuntimeError { message } => format!("RUNTIMEERR({message})"),
    }
}

/// Run `function` in `entry`'s closure via the uncached single-entry path.
fn uncached_outcome(roots: &[String], entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("uncached resolve");
    let ctx = make_eval_context(&graph, si);
    outcome_tag(&run_claim(&ctx, function))
}

/// Run `function` in `entry`'s closure via the cached multi-entry index.
fn cached_outcome(index: &cli_run::MultiEntryIndex, entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_with_index(index, entry).expect("cached resolve");
    let ctx = make_eval_context(&graph, si);
    outcome_tag(&run_claim(&ctx, function))
}

#[test]
fn typed_module_cache_preserves_witness_verdicts_byte_identical() {
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

    // Two distinct modules declaring the SAME unqualified item name `val` with
    // DIFFERENT values — a name-keyed cache must not alias them.
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    // A module shared by BOTH entries — reused from the cache by the second.
    let common = "module test.common\nfn base() -> Int { 100 }\n";

    // entry_a pulls shared1.val (=10) + common.base (=100): true witness + a
    // deliberately-false witness so the FAIL verdict is exercised too.
    let entry_a = "module test.a\n\
        import test.shared1 { val }\n\
        import test.common { base }\n\
        fn witness_a_true() -> Bool { (val() + base()) == 110 }\n\
        fn witness_a_false() -> Bool { val() == 999 }\n";
    // entry_b pulls shared2.val (=20) + common.base (=100): if the cache aliased
    // `val` by item name, witness_b_true would see 10 and flip to false.
    let entry_b = "module test.b\n\
        import test.shared2 { val }\n\
        import test.common { base }\n\
        fn witness_b_true() -> Bool { (val() + base()) == 120 }\n";

    fs::write(dir.join("shared1.dag"), shared1).expect("write shared1");
    fs::write(dir.join("shared2.dag"), shared2).expect("write shared2");
    fs::write(dir.join("common.dag"), common).expect("write common");
    fs::write(dir.join("entry_a.dag"), entry_a).expect("write entry_a");
    fs::write(dir.join("entry_b.dag"), entry_b).expect("write entry_b");

    let roots = vec![dir.to_string_lossy().into_owned()];
    let a_path = dir.join("entry_a.dag").to_string_lossy().into_owned();
    let b_path = dir.join("entry_b.dag").to_string_lossy().into_owned();

    // Cached path: ONE index, resolve A then B so B reuses test.common (and any
    // other overlap) from the warm cache.
    let index = build_multi_entry_index(&roots);
    let cases = [
        (&a_path, "witness_a_true", "PASS"),
        (&a_path, "witness_a_false", "FAIL"),
        (&b_path, "witness_b_true", "PASS"),
    ];
    for (entry, function, expected) in cases {
        let cached = cached_outcome(&index, entry, function);
        let uncached = uncached_outcome(&roots, entry, function);
        assert_eq!(
            cached, uncached,
            "cached vs uncached verdict diverged for {function} in {entry}"
        );
        assert_eq!(
            cached, expected,
            "unexpected verdict for {function} in {entry}"
        );
    }

    // Resolve A a second time through the same (now-warm) index: A's own modules
    // are served entirely from the cache; the verdict must be unchanged.
    let warm_a = cached_outcome(&index, &a_path, "witness_a_true");
    assert_eq!(warm_a, "PASS", "warm-cache re-resolve of A flipped its verdict");

    let _ = fs::remove_dir_all(&dir);
}
