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
    build_multi_entry_index, build_multi_entry_index_with_shared_caches, make_eval_context,
    new_shared_typecheck_caches, reset_typecheck_compute_count, resolve_entry_graph,
    resolve_entry_with_index, run_claim, typecheck_compute_count,
    with_typecheck_compute_count_receipt, workspace_root, ClaimOutcome, MultiEntryIndex,
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
fn union_outcome(index: &MultiEntryIndex, entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_with_index(index, entry).expect("union-view resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

/// §6.2 — once-per-node counter: the union resolve typechecks each distinct module exactly
/// once. The receipt is the enforced form of "resolve cost ≤ 1× union closure, never N×".
#[test]
fn union_resolve_typechecks_each_node_once() {
    with_typecheck_compute_count_receipt(|| {
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
    });
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

/// The discovery input-size axis (`DiscoverySummary::roster_closure_nodes`) must be a property of
/// the SOURCE closure, not of the resolving thread's typecheck cache.
///
/// RED control against the counter-based reading this replaced. `typecheck_compute_count()` counts
/// cache misses and is never reset in production, so a prior resolve on the same thread deflates it
/// for the entry that follows. That prior resolve is not hypothetical: on the `width == 1` discovery
/// path `floor_diff_edits_from_line_ranges` resolves every changed file on the discovery thread
/// before the roster rows, and `floor_skip_discovery_witness` runs discovery three times in one
/// thread. Here `entry_a` stands in for that prior work. The counter moves; the closure must not.
#[test]
fn roster_closure_count_is_independent_of_thread_cache_warmth() {
    with_typecheck_compute_count_receipt(|| {
        let fx = Fixture::new("warmth-independence");

        // The same fold `run_discovery_rows` applies to every graph it resolves.
        macro_rules! closure_of {
            ($graph:expr, $si:expr) => {
                $graph
                    .modules
                    .iter()
                    .map(|m| {
                        v1_compiler::v1_std_core::authored_name_at($si.clone(), m.module.clone())
                    })
                    .collect::<std::collections::BTreeSet<String>>()
            };
        }

        // COLD: entry_b resolved against a fresh index, nothing typechecked before it.
        let idx_cold = build_multi_entry_index(&fx.roots);
        reset_typecheck_compute_count();
        let (graph_cold, si_cold) =
            resolve_entry_with_index(&idx_cold, &fx.entry_b).expect("cold b");
        let counter_cold = typecheck_compute_count();
        let closure_cold: std::collections::BTreeSet<String> = closure_of!(graph_cold, si_cold);

        // WARM: a prior entry resolves on the SAME index/thread first, then entry_b.
        let idx_warm = build_multi_entry_index(&fx.roots);
        reset_typecheck_compute_count();
        resolve_entry_with_index(&idx_warm, &fx.entry_a).expect("prior same-thread resolve");
        let (graph_warm, si_warm) =
            resolve_entry_with_index(&idx_warm, &fx.entry_b).expect("warm b");
        let counter_warm = typecheck_compute_count();
        let closure_warm: std::collections::BTreeSet<String> = closure_of!(graph_warm, si_warm);

        assert!(
            !closure_cold.is_empty(),
            "fixture must resolve a non-empty closure, else this control proves nothing"
        );
        // The defect, made visible: the counter after the same measurement window differs purely
        // because of what the thread happened to resolve earlier. Reading it as a closure size is the
        // bug this test guards. (Cumulative: a's closure is folded in, so warm > cold.)
        assert!(
            counter_warm > counter_cold,
            "precondition: the compute counter must be contaminated by the prior resolve \
         (cold={counter_cold}, warm={counter_warm}) — otherwise this control is not discriminating"
        );
        // The property that must hold: the graph-derived closure is identical either way.
        assert_eq!(
            closure_warm, closure_cold,
            "roster_closure_nodes must count the union closure of the resolved graphs, which is \
         independent of what this thread typechecked earlier"
        );
    });
}

/// C1-prep (cross-worker-typecheck-share-design.md §4.1): `typecheck_compute_count` is a
/// process-wide atomic — prerequisite for summing misses across floor workers once the
/// shared typed_module_cache lands (`Rc`→`Arc` migration). Private per-thread indexes
/// today; the counter still accumulates across threads.
#[test]
fn typecheck_compute_count_accumulates_across_threads() {
    with_typecheck_compute_count_receipt(|| {
        let fx = Fixture::new("process-wide-counter");
        reset_typecheck_compute_count();

        let roots_a = fx.roots.clone();
        let entry_a = fx.entry_a.clone();
        std::thread::spawn(move || {
            let index = build_multi_entry_index(&roots_a);
            resolve_entry_with_index(&index, &entry_a).expect("thread resolve a");
        })
        .join()
        .expect("thread a join");

        let after_a = typecheck_compute_count();
        assert!(after_a > 0, "first thread must record typecheck computes");

        let roots_b = fx.roots.clone();
        let entry_b = fx.entry_b.clone();
        std::thread::spawn(move || {
            let index = build_multi_entry_index(&roots_b);
            resolve_entry_with_index(&index, &entry_b).expect("thread resolve b");
        })
        .join()
        .expect("thread b join");

        assert!(
        typecheck_compute_count() > after_a,
        "second thread's computes must accumulate into the process-wide counter (got {} after {}, now {})",
        after_a,
        after_a,
        typecheck_compute_count()
    );
    });
}

/// S2a increment C — process once-per-node across workers: two threads sharing ONE
/// `SharedTypecheckCaches` must not re-pay the overlapping prefix (u.common).
#[test]
fn cross_worker_shared_typecheck_cache_process_once_per_node() {
    with_typecheck_compute_count_receipt(|| {
        let fx = Fixture::new("cross-worker-once");
        let shared = new_shared_typecheck_caches();
        reset_typecheck_compute_count();

        let idx_a = build_multi_entry_index(&fx.roots);
        reset_typecheck_compute_count();
        resolve_entry_with_index(&idx_a, &fx.entry_a).expect("private cold a");
        let cold_a = typecheck_compute_count();

        let idx_b = build_multi_entry_index(&fx.roots);
        reset_typecheck_compute_count();
        resolve_entry_with_index(&idx_b, &fx.entry_b).expect("private cold b");
        let cold_b = typecheck_compute_count();

        reset_typecheck_compute_count();
        let shared_a = shared.clone();
        let roots_a = fx.roots.clone();
        let entry_a = fx.entry_a.clone();
        std::thread::spawn(move || {
            let index = build_multi_entry_index_with_shared_caches(&roots_a, shared_a);
            resolve_entry_with_index(&index, &entry_a).expect("shared thread a");
        })
        .join()
        .expect("thread a join");
        let after_a = typecheck_compute_count();

        let shared_b = shared.clone();
        let roots_b = fx.roots.clone();
        let entry_b = fx.entry_b.clone();
        std::thread::spawn(move || {
            let index = build_multi_entry_index_with_shared_caches(&roots_b, shared_b);
            resolve_entry_with_index(&index, &entry_b).expect("shared thread b");
        })
        .join()
        .expect("thread b join");
        let union_total = typecheck_compute_count();

        assert_eq!(
            after_a, cold_a,
            "first entry against the shared store pays exactly its private closure"
        );
        let b_added = union_total - after_a;
        assert!(
            b_added < cold_b,
            "second worker's incremental cost ({b_added}) must be below its private closure \
             ({cold_b}) — shared prefix is not re-paid across workers"
        );
        assert!(
            union_total < cold_a + cold_b,
            "cross-worker union ({union_total}) must be < sum of private closures ({})",
            cold_a + cold_b
        );
    });
}

/// S2a increment C — cross-worker purity: shared-store verdicts match private resolve.
#[test]
fn cross_worker_shared_typecheck_cache_purity() {
    let fx = Fixture::new("cross-worker-purity");
    let witnesses: [(&str, &str, &str); 3] = [
        (&fx.entry_a, "wit_a_pass", "PASS"),
        (&fx.entry_a, "wit_a_fail", "FAIL"),
        (&fx.entry_b, "wit_b_pass", "PASS"),
    ];
    for (entry, func, expected) in witnesses {
        assert_eq!(
            private_outcome(&fx.roots, entry, func),
            expected,
            "private baseline for {func}"
        );
    }

    let shared = new_shared_typecheck_caches();
    let shared_a = shared.clone();
    let roots_a = fx.roots.clone();
    let entry_a = fx.entry_a.clone();
    std::thread::spawn(move || {
        let index = build_multi_entry_index_with_shared_caches(&roots_a, shared_a);
        resolve_entry_with_index(&index, &entry_a).expect("warm a on worker 1");
    })
    .join()
    .expect("worker 1 join");

    let shared_b = shared.clone();
    let roots_b = fx.roots.clone();
    let entry_b = fx.entry_b.clone();
    std::thread::spawn(move || {
        let index = build_multi_entry_index_with_shared_caches(&roots_b, shared_b);
        resolve_entry_with_index(&index, &entry_b).expect("warm b on worker 2");
    })
    .join()
    .expect("worker 2 join");

    let index = build_multi_entry_index_with_shared_caches(&fx.roots, shared);
    for (entry, func, expected) in witnesses {
        assert_eq!(
            union_outcome(&index, entry, func),
            expected,
            "cross-worker shared-store verdict for {func}"
        );
    }
}
