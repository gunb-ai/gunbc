//! RESIDUAL after floor-drain retention lane: catalog/model witnesses live in
//! src/v2/test/claim/manual/floor_drain_retention_test.dag (InProcess typed_module_cache,
//! Never mechanism vs ScopeExit projection). The tests below exercise the pinned v1 harness:
//! host-budget entry cap, counted eviction receipts, retention snapshots, and cached-vs-cold
//! purity under cap — runtime facts with no .dag consumer yet.
use std::fs;
use std::sync::Mutex;

use crate::helpers::workspace_root;
use v1_compiler::cli_run::{
    self, build_multi_entry_index, index_retention_snapshot, make_eval_context,
    resolve_entry_graph, resolve_entry_with_index, run_claim, typed_cache_evictions_for_test,
    typed_module_cache_cap_for_test, typed_module_cache_len_for_test,
    typed_module_cache_max_entries, ClaimOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

static CAP_ENV_MUTEX: Mutex<()> = Mutex::new(());

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

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = workspace_root()
        .join("target")
        .join(format!("gunbc-floor-drain-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_small_cap_fixture(dir: &std::path::Path) -> (Vec<String>, String, String) {
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_a_true() -> Bool { (unbox(boxed(val())) + 0) == 10 }\n";
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        fn witness_b_true() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("shared2.dag", shared2),
        ("extra.dag", extra),
        ("entry_a.dag", entry_a),
        ("entry_b.dag", entry_b),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }

    let roots = vec![dir.to_string_lossy().into_owned()];
    let a = dir.join("entry_a.dag").to_string_lossy().into_owned();
    let b = dir.join("entry_b.dag").to_string_lossy().into_owned();
    (roots, a, b)
}

#[test]
fn typed_module_cache_max_entries_honors_env_override() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "42");
    assert_eq!(typed_module_cache_max_entries(), 42);
    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

#[test]
fn typed_module_cache_max_entries_env_probe_clamps_to_ceil() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "999999");
    assert_eq!(typed_module_cache_max_entries(), 4_000);
    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

#[test]
fn typed_module_cache_max_entries_malformed_override_falls_back_to_derived() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "not-a-number");
    let cap = typed_module_cache_max_entries();
    assert!(
        (100..=4_000).contains(&cap),
        "malformed override must fall through to derived clamp, got {cap}"
    );
    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

#[test]
fn index_retention_snapshot_reports_empty_index_shell() {
    let index = build_multi_entry_index(&[]);
    let snap = index_retention_snapshot(&index);
    assert_eq!(snap.typed_module_cache_entries, 0);
    assert_eq!(snap.parse_cache_entries, 0);
    assert_eq!(snap.resolved_graph_memo_entries, 0);
    assert_eq!(snap.typed_cache_evictions, 0);
}

#[test]
fn typed_module_cache_entry_cap_evicts_with_counted_receipt() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    let dir = temp_dir("cap");
    let (roots, a, b) = write_small_cap_fixture(&dir);

    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "2");
    let index = build_multi_entry_index(&roots);
    resolve_entry_with_index(&index, &a).expect("resolve a");
    resolve_entry_with_index(&index, &b).expect("resolve b");

    assert!(
        typed_module_cache_len_for_test(&index) <= 2,
        "typed_module_cache must stay under the modeled cap (got {})",
        typed_module_cache_len_for_test(&index)
    );
    assert!(
        typed_cache_evictions_for_test(&index) > 0,
        "evictions must be counted when the cap is exceeded"
    );

    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
    let _ = fs::remove_dir_all(&dir);
}

/// 2026-07-21 fleet OOM fix: `typed_module_cache_cap` samples the host budget
/// exactly ONCE per index lifetime — a run-start fact, never re-read per
/// insert. This is the by-execution witness that the sampled-once accessor
/// is actually stable: change the "signal" (here, the env-override probe,
/// standing in for the live host-shared MemAvailable reading that moved
/// mid-run pre-fix) after the index has sampled its cap once, and confirm
/// the sampled cap does not follow it.
#[test]
fn typed_module_cache_cap_sampled_once_per_index_stays_stable_despite_signal_drift() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "150");
    let index = build_multi_entry_index(&[]);

    let first = typed_module_cache_cap_for_test(&index);
    assert_eq!(
        first, 150,
        "index must sample the cap that was live at first use"
    );

    // Simulate the host signal moving mid-run (this is exactly the class of
    // motion `/proc/meminfo MemAvailable` exhibited under co-tenant pressure
    // in the 2026-07-21 incident).
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "3000");
    let second = typed_module_cache_cap_for_test(&index);
    assert_eq!(
        second, first,
        "cap must stay fixed for this index's lifetime even as the underlying signal moves"
    );

    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

/// RED control: reproduces the pre-fix defect shape. The un-cached, pure
/// derivation (`typed_module_cache_max_entries`, still used by direct
/// env-override tests above and kept for that purpose) DOES track a moving
/// signal on every call — that tracking, invoked from every cache insert,
/// is the mechanism that produced the eviction storm (cap 294→227 across
/// 2300+ evictions, `claim_executor` exit 137). This test asserts the old
/// call pattern's oscillation is real and observable, so it stays a live
/// discriminator: were `typed_module_cache_cap` ever accidentally reverted
/// to call the uncached function per-insert, this divergence-producing
/// signal motion is what would resume driving it.
#[test]
fn uncached_derivation_tracks_moving_signal_the_sampled_once_accessor_must_not() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "300");
    let readings: Vec<usize> = ["300", "227", "294", "204", "325"]
        .iter()
        .map(|v| {
            std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", v);
            typed_module_cache_max_entries()
        })
        .collect();
    assert_eq!(
        readings,
        vec![300, 227, 294, 204, 325],
        "uncached derivation must track every signal move — this is the storm mechanism the \
         sampled-once accessor exists to eliminate at runtime call sites"
    );

    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

#[test]
fn typed_module_cache_under_entry_cap_matches_cold_oracle_in_every_order() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    let dir = temp_dir("cap-purity");
    let (roots, a, b) = write_small_cap_fixture(&dir);

    let witnesses = [
        (&a, "witness_a_true", "PASS"),
        (&b, "witness_b_true", "PASS"),
    ];

    for (entry, f, expected) in witnesses {
        let cold = cold_oracle(&roots, entry, f);
        assert_eq!(cold, expected, "cold oracle unexpected for {f}");
    }

    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "2");
    let orders: [&[&str]; 2] = [&[&a, &b], &[&b, &a]];
    for order in orders {
        let index = build_multi_entry_index(&roots);
        for entry in order {
            let _ = resolve_entry_with_index(&index, entry).expect("warm resolve under cap");
        }
        assert!(
            typed_cache_evictions_for_test(&index) > 0,
            "cap=2 must force evictions in order {order:?}"
        );
        for (entry, f, expected) in witnesses {
            let got = cached(&index, entry, f);
            assert_eq!(
                got, expected,
                "cached verdict for {f} diverged from cold oracle under cap in order {order:?}"
            );
        }
    }

    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
    let _ = fs::remove_dir_all(&dir);
}
