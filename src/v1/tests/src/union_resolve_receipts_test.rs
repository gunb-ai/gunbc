//! Union-resolve S1 receipts (docs/plans/resolver-graph-major-design.md §6).
//!
//! S1 stops fragmenting resolve demand: one process resolves the union of everything it
//! needs against ONE shared index, so the shared std/spec prefix typechecks once per node
//! and every consumer assembles a per-entry view over the shared immutable module facts.
//! These are the two executable oracles that hold the interim contract:
//!
//!  - §6.2 once-per-node counter: a process's resolve cost is ≤ 1× its union closure, never
//!    N× — the minimum upper bound, enforced (typecheck computes == distinct nodes), not
//!    aspired. A private re-resolve sneaking back would push computes above distinct nodes.
//!  - §6.1 byte-identity oracle: the union-view result of an entry equals its private
//!    (fresh, closure-scoped) resolve — same claim outcomes, in every resolve order. Sharing
//!    is sound because module meaning flows one way in a DAG (§3), so evaluation order is
//!    unobservable in any result.
//!
//! The collision-honesty receipt (§6.3) lives beside the guard it exercises
//! (`shared_cache_collision_guard_tests` in cli_run.rs) — the guard is a private fn.
//!
//! Fixtures live under the workspace `target/` dir (not `std::env::temp_dir()`), because the
//! import-closure fact index requires workspace-relative module paths.

use std::fs;

use v1_compiler::cli_run::{
    self, build_multi_entry_index, make_eval_context, reset_typecheck_compute_count,
    resolve_entry_graph, resolve_entry_with_index, run_claim, typecheck_compute_count,
    workspace_root, ClaimOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

/// A 5-module corpus with a deliberately-shared prefix: entries `u.a` and `u.b` both import
/// `u.common`, plus one private leaf each. Closure(a) = {u.a, u.common, u.a1} (3);
/// closure(b) = {u.b, u.common, u.b1} (3); union distinct = 5; overlap = {u.common} = 1.
/// This is the corpus shape at small scale — a big shared std/spec prefix under many entries.
const FIXTURES: &[(&str, &str)] = &[
    ("common.dag", "module u.common\nfn base() -> Int { 10 }\n"),
    ("a1.dag", "module u.a1\nfn av() -> Int { 1 }\n"),
    ("b1.dag", "module u.b1\nfn bv() -> Int { 2 }\n"),
    (
        "entry_a.dag",
        "module u.a\n\
         import u.common { base }\n\
         import u.a1 { av }\n\
         fn wit_a_pass() -> Bool { base() + av() == 11 }\n\
         fn wit_a_fail() -> Bool { av() == 99 }\n",
    ),
    (
        "entry_b.dag",
        "module u.b\n\
         import u.common { base }\n\
         import u.b1 { bv }\n\
         fn wit_b_pass() -> Bool { base() + bv() == 12 }\n",
    ),
];

struct Fixture {
    dir: std::path::PathBuf,
    roots: Vec<String>,
    entry_a: String,
    entry_b: String,
}

impl Fixture {
    fn new(tag: &str) -> Fixture {
        // Unique, workspace-relative fixture root (target/ is gitignored + ephemeral). A
        // process-id + monotonic counter keeps parallel test threads from colliding without
        // needing a wall clock.
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = workspace_root().join("target").join(format!(
            "union-resolve-receipt-{tag}-{}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture dir");
        for (name, src) in FIXTURES {
            fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
        }
        let roots = vec![dir.to_string_lossy().into_owned()];
        let entry_a = dir.join("entry_a.dag").to_string_lossy().into_owned();
        let entry_b = dir.join("entry_b.dag").to_string_lossy().into_owned();
        Fixture {
            dir,
            roots,
            entry_a,
            entry_b,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn outcome_tag(o: &ClaimOutcome) -> String {
    match o {
        ClaimOutcome::Pass => "PASS".to_string(),
        ClaimOutcome::Fail => "FAIL".to_string(),
        ClaimOutcome::NotBool { got } => format!("NOTBOOL({got})"),
        ClaimOutcome::RuntimeError { message } => format!("RUNTIMEERR({message})"),
    }
}

/// Private resolve: a fresh, closure-scoped resolve sharing nothing (the request-major
/// baseline the union replaces).
fn private_outcome(roots: &[String], entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("private resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

/// Union-view resolve: the entry assembled over the shared index (already warmed by the
/// other entries in this process).
fn union_outcome(index: &cli_run::MultiEntryIndex, entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_with_index(index, entry).expect("union-view resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

/// §6.2 — once-per-node counter: the union resolve typechecks each distinct module exactly
/// once. The receipt is the enforced form of "resolve cost ≤ 1× union closure, never N×".
#[test]
fn union_resolve_typechecks_each_node_once() {
    let fx = Fixture::new("once-per-node");

    // Cold private closures — the request-major baseline, one fresh index each. cold_a and
    // cold_b are |closure(a)| and |closure(b)|; each re-typechecks the shared prefix privately.
    let idx_a = build_multi_entry_index(&fx.roots);
    reset_typecheck_compute_count();
    resolve_entry_with_index(&idx_a, &fx.entry_a).expect("cold resolve a");
    let cold_a = typecheck_compute_count();

    let idx_b = build_multi_entry_index(&fx.roots);
    reset_typecheck_compute_count();
    resolve_entry_with_index(&idx_b, &fx.entry_b).expect("cold resolve b");
    let cold_b = typecheck_compute_count();

    assert!(
        cold_a > 0 && cold_b > 0,
        "each closure must typecheck at least one module (cold_a={cold_a}, cold_b={cold_b})"
    );

    // Union — ONE shared index. Resolving a then b: the shared prefix (u.common, and any
    // kernel modules) is typechecked once; b only pays for its private leaf.
    let idx = build_multi_entry_index(&fx.roots);
    reset_typecheck_compute_count();
    resolve_entry_with_index(&idx, &fx.entry_a).expect("union resolve a");
    let union_after_a = typecheck_compute_count();
    resolve_entry_with_index(&idx, &fx.entry_b).expect("union resolve b");
    let union_after_b = typecheck_compute_count();

    // A alone against the shared index costs exactly its private closure — the shared index
    // adds no work, it only removes duplication downstream.
    assert_eq!(
        union_after_a, cold_a,
        "first entry in the shared index computes exactly its own closure"
    );
    // b adds strictly fewer computes than its private closure: the overlap (>=1: u.common)
    // was already typechecked by a and is NOT recomputed.
    let b_added = union_after_b - union_after_a;
    assert!(
        b_added < cold_b,
        "b's incremental cost ({b_added}) must be below its private closure ({cold_b}) — the \
         shared prefix is not re-paid"
    );
    // The core minimum-upper-bound contract: the union costs strictly less than N private
    // resolves. Each shared node is paid once across the whole process, never once per entry.
    assert!(
        union_after_b < cold_a + cold_b,
        "union resolve cost ({union_after_b}) must be < sum of private closures \
         ({}) — resolve cost ≤ 1× union closure, not N×",
        cold_a + cold_b
    );

    // Once-per-node, made unrepresentable: re-resolving already-resolved entries computes
    // NOTHING new. A private re-resolve sneaking back would bump the counter here.
    resolve_entry_with_index(&idx, &fx.entry_a).expect("re-resolve a");
    resolve_entry_with_index(&idx, &fx.entry_b).expect("re-resolve b");
    assert_eq!(
        typecheck_compute_count(),
        union_after_b,
        "once-per-node: re-resolving computes zero new typechecks — the node is in the schedule once"
    );
}

/// §6.1 — byte-identity oracle: the union-view result of an entry equals its private
/// closure-scoped resolve, in every resolve order. Sharing is a pure fact of the source
/// snapshot (§3), so co-residence and order are unobservable in any claim outcome.
#[test]
fn union_view_result_equals_private_resolve_in_every_order() {
    let fx = Fixture::new("byte-identity");

    // (entry, function, expected) — a discriminating sample: passes AND a fail, so a wrong
    // shared resolution (e.g. u.common's `base` bleeding a wrong value) flips a verdict.
    let witnesses: [(&str, &str, &str); 3] = [
        (&fx.entry_a, "wit_a_pass", "PASS"),
        (&fx.entry_a, "wit_a_fail", "FAIL"),
        (&fx.entry_b, "wit_b_pass", "PASS"),
    ];

    // The private (request-major) baseline the union must match.
    for (entry, func, expected) in witnesses {
        assert_eq!(
            private_outcome(&fx.roots, entry, func),
            expected,
            "private-resolve baseline unexpected for {func}"
        );
    }

    // Every resolve order of the shared index must produce byte-identical verdicts to the
    // private baseline — the whole point of "isolation = purity, not privacy" (§3).
    let orders: [&[&str]; 3] = [
        &[&fx.entry_a, &fx.entry_b],
        &[&fx.entry_b, &fx.entry_a],
        &[&fx.entry_b, &fx.entry_a, &fx.entry_a],
    ];
    for order in orders {
        let index = build_multi_entry_index(&fx.roots);
        for entry in order {
            resolve_entry_with_index(&index, entry).expect("warm the shared index");
        }
        for (entry, func, expected) in witnesses {
            assert_eq!(
                union_outcome(&index, entry, func),
                expected,
                "union-view verdict for {func} diverged from the private resolve in order {order:?}"
            );
        }
    }
}
