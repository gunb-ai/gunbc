use std::fs;
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

static CACHE_ENV_MUTEX: Mutex<()> = Mutex::new(());

struct CacheEnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
}

impl CacheEnvGuard {
    fn set(cache_dir: &std::path::Path) -> Self {
        let lock = CACHE_ENV_MUTEX
            .lock()
            .expect("GUNBC_RESOLVED_GRAPH_CACHE_DIR env mutex poisoned");
        let prev = std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR");
        std::env::set_var(
            "GUNBC_RESOLVED_GRAPH_CACHE_DIR",
            cache_dir.to_string_lossy().as_ref(),
        );
        Self { _lock: lock, prev }
    }
}

impl Drop for CacheEnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => std::env::set_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR", v),
            None => std::env::remove_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR"),
        }
    }
}

use v1_compiler::cli_run::{
    build_multi_entry_index, load_sources_for_entry, make_eval_context, resolve_entry_graph,
    resolve_entry_with_index, run_claim, ClaimOutcome,
};
use v1_compiler::resolved_graph_cache::{
    build_valid_artifact_bytes, decode_count, derive_subject_digest, lookup,
    subject_digest_for_closure, write_raw_artifact_for_test, CacheLookupResult, CacheRejectReason,
    KeyInputMaterials,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = crate::helpers::workspace_root()
        .join("target")
        .join(format!(
            "gunbc-rg-cache-{label}-{}-{}",
            std::process::id(),
            nanos
        ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn outcome_tag(o: &ClaimOutcome) -> String {
    match o {
        ClaimOutcome::Pass => "PASS".to_string(),
        ClaimOutcome::Fail => "FAIL".to_string(),
        ClaimOutcome::NotBool { got } => format!("NOTBOOL({got})"),
        ClaimOutcome::RuntimeError { message } => format!("RUNTIMEERR({message})"),
    }
}

fn write_fixture(dir: &std::path::Path) -> (Vec<String>, String, String, String) {
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
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        fn witness_b_true() -> Bool { unbox(boxed(val())) == 20 }\n";
    let entry_c = "module test.c\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_c_true() -> Bool { unbox(boxed(val() + 5)) == 15 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("shared2.dag", shared2),
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
    (roots, a, b, c)
}

fn with_cache_env<T, F: FnOnce() -> T>(cache_dir: &std::path::Path, f: F) -> T {
    let _guard = CacheEnvGuard::set(cache_dir);
    f()
}

fn cold_oracle(roots: &[String], entry: &str, function: &str) -> String {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("cold resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    outcome_tag(&run_claim(&ctx, function))
}

fn cached_verdict(
    roots: &[String],
    entry: &str,
    function: &str,
    cache_dir: &std::path::Path,
) -> String {
    with_cache_env(cache_dir, || {
        let index = build_multi_entry_index(roots);
        let (graph, si) = resolve_entry_with_index(&index, entry).expect("cached resolve");
        let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
        outcome_tag(&run_claim(&ctx, function))
    })
}

fn cached_resolve_err(roots: &[String], entry: &str, cache_dir: &std::path::Path) -> String {
    with_cache_env(cache_dir, || {
        let index = build_multi_entry_index(roots);
        resolve_entry_with_index(&index, entry).expect_err("expected provider refusal")
    })
}

#[test]
fn cross_process_cache_matches_cold_oracle_corpus() {
    let dir = temp_dir("eq");
    let (roots, a, b, c) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let witnesses = [
        (&a, "witness_a_true", "PASS"),
        (&a, "witness_a_false", "FAIL"),
        (&b, "witness_b_true", "PASS"),
        (&c, "witness_c_true", "PASS"),
    ];

    for (entry, f, expected) in witnesses {
        assert_eq!(
            cold_oracle(&roots, entry, f),
            expected,
            "cold oracle for {f}"
        );
    }

    let orders: [&[&str]; 3] = [&[&a, &b, &c], &[&c, &b, &a], &[&b, &a, &c]];
    for (i, order) in orders.iter().enumerate() {
        let order_cache = cache_dir.join(format!("order-{i}"));
        fs::create_dir_all(&order_cache).expect("order cache");
        for entry in *order {
            let _ = cached_verdict(&roots, entry, "witness_a_true", &order_cache);
        }
        for (entry, _f, _expected) in witnesses {
            let err = cached_resolve_err(&roots, entry, &order_cache);
            assert!(
                err.contains("provider refused incomplete disk hit"),
                "v1 disk artifact reuse must stop the line until union lands (part a): {err}"
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn poisoned_hit_rejected_on_content_digest_mismatch() {
    let dir = temp_dir("poison");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let valid = build_valid_artifact_bytes(&subject, &graph, si.as_ref()).expect("valid bytes");
    let mut poisoned = valid;
    if let Some(last) = poisoned.last_mut() {
        *last ^= 0xff;
    }
    write_raw_artifact_for_test(&cache_dir, &subject, &poisoned).expect("poison write");

    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::ContentDigestMismatch) => {}
        other => panic!("expected ContentDigestMismatch RejectedHit, got {other:?}"),
    }

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let (recomputed, si2) =
            resolve_entry_with_index(&index, &a).expect("recompute after poison");
        let ctx = make_eval_context(&recomputed, si2, ExecutionMode::Wet);
        assert_eq!(
            outcome_tag(&run_claim(&ctx, "witness_a_true")),
            "PASS",
            "poisoned hit must fall through to fresh resolve"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn poisoned_hit_rejected_on_subject_digest_mismatch() {
    let dir = temp_dir("poison-subject");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let mut poisoned =
        build_valid_artifact_bytes(&subject, &graph, si.as_ref()).expect("valid bytes");
    let subject_off = 8 + 4;
    poisoned[subject_off] ^= 0xff;
    write_raw_artifact_for_test(&cache_dir, &subject, &poisoned).expect("poison write");

    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let (recomputed, si2) =
            resolve_entry_with_index(&index, &a).expect("recompute after subject poison");
        let ctx = make_eval_context(&recomputed, si2, ExecutionMode::Wet);
        assert_eq!(
            outcome_tag(&run_claim(&ctx, "witness_a_true")),
            "PASS",
            "subject poison must fall through to fresh resolve"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn concurrent_resolve_write_once_no_torn_read() {
    let dir = temp_dir("race");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = Arc::new(dir.join("cache"));
    fs::create_dir_all(cache_dir.as_path()).expect("cache dir");
    let barrier = Arc::new(Barrier::new(2));
    let roots_a = roots.clone();
    let entry = a.clone();
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let _env = CacheEnvGuard::set(cache_dir.as_path());
    let t1 = thread::spawn(move || {
        b1.wait();
        let index = build_multi_entry_index(&roots_a);
        match resolve_entry_with_index(&index, &entry) {
            Ok((graph, si)) => {
                let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
                outcome_tag(&run_claim(&ctx, "witness_a_true"))
            }
            Err(e) => format!("RESOLVEERR({e})"),
        }
    });
    let roots_b = roots.clone();
    let a_b = a.clone();
    let t2 = thread::spawn(move || {
        b2.wait();
        let index = build_multi_entry_index(&roots_b);
        match resolve_entry_with_index(&index, &a_b) {
            Ok((graph, si)) => {
                let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
                outcome_tag(&run_claim(&ctx, "witness_a_true"))
            }
            Err(e) => format!("RESOLVEERR({e})"),
        }
    });

    let v1 = t1.join().expect("t1 join");
    let v2 = t2.join().expect("t2 join");
    for (label, verdict) in [("t1", &v1), ("t2", &v2)] {
        match verdict.as_str() {
            "PASS" => {}
            other if other.contains("provider refused incomplete disk hit") => {}
            other => panic!("{label} unexpected verdict: {other}"),
        }
    }
    assert!(
        v1 == "PASS" || v2 == "PASS",
        "at least one concurrent resolve must cold-build: v1={v1} v2={v2}"
    );

    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    match lookup(cache_dir.as_path(), &subject) {
        CacheLookupResult::Hit(_) => {}
        other => panic!("expected readable cache artifact after race, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn key_mismatch_forces_miss_not_stale_hit() {
    let dir = temp_dir("key");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let _ = resolve_entry_with_index(&index, &a).expect("warm cache");
    });

    let sources_before = load_sources_for_entry(&roots, &a).expect("sources before");
    let digest_before = subject_digest_for_closure(&sources_before);

    let mutated = dir.join("entry_a.dag");
    let content = fs::read_to_string(&mutated).expect("read entry");
    fs::write(
        &mutated,
        format!("{content}\nfn perturb_marker() -> Int {{ 0 }}\n"),
    )
    .expect("perturb entry");

    let sources_after = load_sources_for_entry(&roots, &a).expect("sources after");
    let digest_after = subject_digest_for_closure(&sources_after);
    assert_ne!(
        digest_after, digest_before,
        "perturb must change subject digest"
    );

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let (graph, si) = resolve_entry_with_index(&index, &a).expect("resolve after perturb");
        let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
        assert_eq!(
            outcome_tag(&run_claim(&ctx, "witness_a_true")),
            "PASS",
            "perturbed closure must still resolve correctly (miss path)"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cross_process_child_worker() {
    if std::env::var("GUNBC_RG_CACHE_CHILD").ok().as_deref() != Some("1") {
        return;
    }
    let roots = std::env::var("GUNBC_RG_CACHE_ROOT").expect("root");
    let entry = std::env::var("GUNBC_RG_CACHE_ENTRY").expect("entry");
    let cache = std::env::var("GUNBC_RESOLVED_GRAPH_CACHE_DIR").expect("cache");
    let verdict_path = std::env::var("GUNBC_RG_CACHE_VERDICT").expect("verdict path");
    let verdict = with_cache_env(std::path::Path::new(&cache), || {
        let index = build_multi_entry_index(&[roots]);
        let (graph, si) = resolve_entry_with_index(&index, &entry).expect("child resolve");
        let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
        outcome_tag(&run_claim(&ctx, "witness_a_true"))
    });
    fs::write(&verdict_path, verdict).expect("write verdict");
}

fn spawn_cache_child(
    exe: &std::path::Path,
    roots: &str,
    entry: &str,
    cache: &std::path::Path,
    verdict_path: &std::path::Path,
) -> ExitStatus {
    Command::new(exe)
        .env("GUNBC_RG_CACHE_CHILD", "1")
        .env("GUNBC_RG_CACHE_ROOT", roots)
        .env("GUNBC_RG_CACHE_ENTRY", entry)
        .env(
            "GUNBC_RESOLVED_GRAPH_CACHE_DIR",
            cache.to_string_lossy().as_ref(),
        )
        .env(
            "GUNBC_RG_CACHE_VERDICT",
            verdict_path.to_string_lossy().as_ref(),
        )
        .arg("resolve_cross_process_cache_test::cross_process_child_worker")
        .arg("--exact")
        .arg("--nocapture")
        .status()
        .expect("spawn child")
}

#[test]
fn two_processes_share_cache_without_torn_read() {
    let dir = temp_dir("xproc");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    let exe = std::env::current_exe().expect("current exe");
    let roots_s = roots[0].clone();
    let verdict1 = dir.join("verdict1.txt");
    let verdict2 = dir.join("verdict2.txt");

    let s1 = spawn_cache_child(&exe, &roots_s, &a, &cache_dir, &verdict1);
    let s2 = spawn_cache_child(&exe, &roots_s, &a, &cache_dir, &verdict2);
    assert!(s1.success(), "child1 status: {s1:?}");
    assert!(s2.success(), "child2 status: {s2:?}");
    let out1 = fs::read_to_string(&verdict1).expect("verdict1");
    let out2 = fs::read_to_string(&verdict2).expect("verdict2");
    assert_eq!(out1.trim(), "PASS", "child1 verdict");
    assert_eq!(out2.trim(), "PASS", "child2 verdict");

    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::Hit(_) => {}
        other => panic!("expected cache hit after cross-process warm, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

const CLOSURE_A: &str = "aaaaaaaaaaaaaaaa";
const CLOSURE_B: &str = "bbbbbbbbbbbbbbbb";
const TRANSFORM_1: &str = "1111111111111111";
const TRANSFORM_2: &str = "2222222222222222";

#[test]
fn key_changes_when_transform_axis_changes() {
    let k1 = derive_subject_digest(&KeyInputMaterials::new(
        CLOSURE_A.to_string(),
        TRANSFORM_1.to_string(),
    ));
    let k2 = derive_subject_digest(&KeyInputMaterials::new(
        CLOSURE_A.to_string(),
        TRANSFORM_2.to_string(),
    ));
    assert_ne!(
        k1, k2,
        "transform (toolchain) axis must be keyed: changing content(transform) must change the key"
    );
}

#[test]
fn key_changes_when_closure_axis_changes() {
    let k1 = derive_subject_digest(&KeyInputMaterials::new(
        CLOSURE_A.to_string(),
        TRANSFORM_1.to_string(),
    ));
    let k2 = derive_subject_digest(&KeyInputMaterials::new(
        CLOSURE_B.to_string(),
        TRANSFORM_1.to_string(),
    ));
    assert_ne!(
        k1, k2,
        "closure-subject axis must be keyed: changing the closure content must change the key"
    );
}

#[test]
fn key_is_deterministic_in_its_axes() {
    let m1 = KeyInputMaterials::new(CLOSURE_A.to_string(), TRANSFORM_1.to_string());
    let m2 = KeyInputMaterials::new(CLOSURE_A.to_string(), TRANSFORM_1.to_string());
    assert_eq!(
        derive_subject_digest(&m1),
        derive_subject_digest(&m2),
        "the key is a pure function of its axes: identical axes ⟹ identical key (no hidden input)"
    );
}

// The ladder's tier ordering at the resolve seam (store fills share — never
// replaces it): the per-process share serves repeats by reference; the store
// would serve a process's first touch of a subject and install into the share
// once the provider can serve a complete artifact (part a). Until then, a v1
// disk hit is typed provider refusal and stops the line — no cold-resolve widen.
#[test]
fn same_subject_resolves_share_one_graph_store_refuses_v1_disk_hit() {
    let dir = temp_dir("share");
    let (roots, a, _b, _c) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    with_cache_env(&cache_dir, || {
        let decodes_before = decode_count();
        let index = build_multi_entry_index(&roots);
        let (g1, _) = resolve_entry_with_index(&index, &a).expect("build resolve");
        let (g2, _) = resolve_entry_with_index(&index, &a).expect("repeat resolve");
        assert!(
            std::rc::Rc::ptr_eq(&g1, &g2),
            "repeat resolve must serve the shared reference, not a rebuild"
        );
        assert_eq!(
            decode_count(),
            decodes_before,
            "the build path must not decode"
        );

        let index2 = build_multi_entry_index(&roots);
        let err = resolve_entry_with_index(&index2, &a).expect_err("v1 store hit must refuse");
        assert!(
            err.contains("provider refused incomplete disk hit"),
            "fresh index must not cold-resolve through v1 disk hit: {err}"
        );
        assert_eq!(
            decode_count(),
            decodes_before,
            "provider refusal must not decode or rebuild"
        );
    });
}
