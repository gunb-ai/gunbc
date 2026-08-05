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

use im::HashMap;
use im::Vector;
use std::rc::Rc;
use v1_compiler::cli_run::{
    build_multi_entry_index, load_sources_for_entry, make_eval_context,
    materialization_provider_ctx_build_count_for_test, provider_bootstrap_store_skip_count,
    provider_ctx_reentrancy_refusal_for_test, reset_typecheck_compute_count,
    resolve_closure_request_key_from_digests, resolve_entry_graph, resolve_entry_with_index,
    resolved_graph_parts_semantic_digest, run_claim, typecheck_compute_count,
    with_typecheck_compute_count_receipt, ClaimOutcome,
};
use v1_compiler::resolved_graph_cache::{
    build_incomplete_v3_artifact_bytes, build_valid_artifact_bytes, closure_content_digest,
    decode_count, derive_subject_digest, encode_resolved_graph_parts, lookup,
    lookup_verified_probe, probe, subject_digest_for_closure, transform_content_digest,
    write as write_resolved_graph_cache, write_raw_artifact_for_test, CacheLookupResult,
    CacheProbeResult, CacheRejectReason, CacheWriteOutcome, KeyInputMaterials,
    UNION_PART_ABSENT_DIGEST,
};
use v1_compiler::v1_interpreter::ExecutionMode;
use v1_compiler::v1_rt::bytes_identity_hash;

fn empty_compile_clean_diags() -> Vector<Rc<v1_compiler::v1_std_core::ErrorNode>> {
    Vector::new()
}

fn provider_keys_for_graph(
    roots: &[String],
    entry: &str,
    graph: &v1_compiler::v1_compiler_infer_items::ResolvedGraph,
    si: &HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>,
    diags: &Vector<Rc<v1_compiler::v1_std_core::ErrorNode>>,
) -> (String, String) {
    let sources = load_sources_for_entry(roots, entry).expect("sources");
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let encoded = encode_resolved_graph_parts(graph, si, diags).expect("encode");
    let request_key = resolve_closure_request_key_from_digests(&closure_digest, &compiler_digest)
        .expect("request key");
    let semantic = resolved_graph_parts_semantic_digest(
        &encoded.graph_digest,
        encoded.graph_bytes.len() as u64,
        &encoded.indices_digest,
        encoded.indices_bytes.len() as u64,
        &encoded.union_digest,
        encoded.union_bytes.len() as u64,
    )
    .expect("semantic digest");
    (request_key, semantic)
}

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
        // Distinct from RUNTIMEERR on purpose: a budget kill is not a runtime fault.
        // Both numbers are rendered so a caller comparing labels cannot mistake the
        // elapsed ceiling for a completed duration.
        ClaimOutcome::TimedOut {
            elapsed_ms,
            budget_ms,
            kind,
        } => format!("TIMEDOUT({} {elapsed_ms}ms>{budget_ms}ms)", kind.label()),
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
            with_cache_env(&order_cache, || {
                let index = build_multi_entry_index(&roots);
                resolve_entry_with_index(&index, entry).expect("warm v2 cache artifact");
            });
        }
        for (entry, f, expected) in witnesses {
            assert_eq!(
                cached_verdict(&roots, entry, f, &order_cache),
                expected,
                "cached oracle for {f} in order-{i}"
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
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let valid = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    let decodes_before = decode_count();
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
        let err = resolve_entry_with_index(&index, &a).expect_err("poisoned hit must refuse");
        assert!(
            err.contains("content digest mismatch") || err.contains("refused poisoned"),
            "poisoned hit must refuse before rebuilding: {err}"
        );
        assert_eq!(
            decode_count(),
            decodes_before,
            "provider refusal must not decode or rebuild"
        );
        assert_eq!(
            outcome_tag(&run_claim(&ctx, "witness_a_true")),
            "PASS",
            "control resolve remains green"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn malformed_header_probe_refuses_as_backend_key_malformed() {
    let dir = temp_dir("malformed-header");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    bytes[0] ^= 0xff;
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("malformed write");

    match probe(&cache_dir, &subject) {
        CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let err = resolve_entry_with_index(&index, &a).expect_err("malformed header must refuse");
        assert!(
            err.contains("backend key malformed"),
            "malformed header must refuse before rebuilding: {err}"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn malformed_header_truncated_read_refuses_as_backend_key_malformed() {
    let dir = temp_dir("truncated-header");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    bytes.truncate(7);
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("malformed write");

    match probe(&cache_dir, &subject) {
        CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let err = resolve_entry_with_index(&index, &a).expect_err("truncated header must refuse");
        assert!(
            err.contains("backend key malformed"),
            "truncated header must refuse before rebuilding: {err}"
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
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut poisoned = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    let subject_off = 8 + 4;
    poisoned[subject_off] ^= 0xff;
    write_raw_artifact_for_test(&cache_dir, &subject, &poisoned).expect("poison write");

    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }

    with_cache_env(&cache_dir, || {
        let index = build_multi_entry_index(&roots);
        let err = resolve_entry_with_index(&index, &a).expect_err("subject poison must refuse");
        assert!(
            err.contains("backend key malformed"),
            "subject poison must refuse before rebuilding: {err}"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn trailing_payload_bytes_refuse_as_backend_key_malformed() {
    let dir = temp_dir("trailing-payload");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    bytes.extend_from_slice(b"extra");
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("trailing write");

    match probe(&cache_dir, &subject) {
        CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed lookup, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn hash_verified_part_decode_failure_refuses_not_miss() {
    const V3_HEADER_LEN: usize = 8 + 4 + 16 + 16 + 16 + 16 + 8 + 3 * (8 + 8 + 16);

    let dir = temp_dir("decode-fail");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    bytes[V3_HEADER_LEN] ^= 0x01;
    let payload_len = u64::from_le_bytes(bytes[76..84].try_into().unwrap()) as usize;
    let graph_len = u64::from_le_bytes(bytes[92..100].try_into().unwrap()) as usize;
    let payload_slice = bytes[V3_HEADER_LEN..V3_HEADER_LEN + payload_len].to_vec();
    let graph_digest = bytes_identity_hash(&payload_slice[0..graph_len]);
    bytes[100..116].copy_from_slice(graph_digest.as_bytes());
    let payload_integrity = bytes_identity_hash(&payload_slice);
    bytes[28..44].copy_from_slice(payload_integrity.as_bytes());
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("corrupt write");

    let decodes_before = decode_count();
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::PartDecodeFailure) => {}
        other => panic!("expected PartDecodeFailure RejectedHit, got {other:?}"),
    }
    assert_eq!(
        decode_count(),
        decodes_before,
        "decode failure must refuse without cold rebuild"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn declared_payload_len_over_cap_refuses_before_large_allocation() {
    let dir = temp_dir("oversized-decl");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    // payload_len field sits after magic+version+four 16-byte digests (offset 76).
    let payload_len_off = 76;
    let bogus_len = u64::MAX;
    bytes[payload_len_off..payload_len_off + 8].copy_from_slice(&bogus_len.to_le_bytes());
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("oversized write");

    match probe(&cache_dir, &subject) {
        CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed lookup, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn part_offset_overflow_refuses_as_backend_key_malformed() {
    let dir = temp_dir("part-offset-overflow");
    let (roots, a, _, _) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve for digest");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let mut bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("valid bytes");
    // Third part descriptor offset field (after magic+version+four digests+payload_len+two parts).
    let part2_offset_off = 76 + 8 + 2 * 24;
    let bogus_offset = u64::MAX - 5;
    bytes[part2_offset_off..part2_offset_off + 8].copy_from_slice(&bogus_offset.to_le_bytes());
    write_raw_artifact_for_test(&cache_dir, &subject, &bytes).expect("overflow write");

    match probe(&cache_dir, &subject) {
        CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed RejectedHit, got {other:?}"),
    }
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {}
        other => panic!("expected BackendKeyMalformed lookup, got {other:?}"),
    }

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
            other if other.contains("provider refused faithful probe") => {}
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
        match resolve_entry_with_index(&index, &entry) {
            Ok((graph, si)) => {
                let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
                outcome_tag(&run_claim(&ctx, "witness_a_true"))
            }
            Err(e) => e,
        }
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
    for (label, verdict) in [("child1", out1.trim()), ("child2", out2.trim())] {
        assert_eq!(
            verdict, "PASS",
            "{label} must resolve green via cold-build or v2 disk hit: {verdict}"
        );
    }

    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::Hit(_) => {}
        other => panic!("expected cache hit after cross-process warm, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

fn build_legacy_v1_probe_artifact(subject: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"gunbgrpc");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(subject.as_bytes());
    bytes.extend_from_slice(&[0u8; 16]);
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes
}

/// Four-arm disposition matrix for the cross-process provider seam:
/// complete v3 hit -> served; absent artifact -> cold compute; legacy v1 ->
/// declared migration cold compute; incomplete provider verdict -> typed refusal
/// with zero cold recompute.
#[test]
fn provider_disposition_four_arm_matrix() {
    let dir = temp_dir("four-arm");
    let (roots, a, _, _) = write_fixture(&dir);
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve fixture");
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let encoded = encode_resolved_graph_parts(&graph, si.as_ref(), &empty_compile_clean_diags())
        .expect("encode");
    let v3_bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("v3 bytes");

    // Arm 2 — absent artifact: cold compute allowed.
    let miss_dir = dir.join("miss-cache");
    fs::create_dir_all(&miss_dir).expect("miss cache dir");
    with_cache_env(&miss_dir, || {
        let index = build_multi_entry_index(&roots);
        let (g1, _) = resolve_entry_with_index(&index, &a).expect("cold resolve");
        let (g2, _) = resolve_entry_with_index(&index, &a).expect("repeat resolve");
        assert!(
            std::rc::Rc::ptr_eq(&g1, &g2),
            "repeat resolve must share memo after cold build"
        );
    });

    // Arm 1 — complete v3 hit: served from disk; decode once on first install only.
    let hit_dir = dir.join("hit-cache");
    fs::create_dir_all(&hit_dir).expect("hit cache dir");
    write_raw_artifact_for_test(&hit_dir, &subject, &v3_bytes).expect("v3 write");
    with_cache_env(&hit_dir, || {
        let decodes_before = decode_count();
        let index = build_multi_entry_index(&roots);
        let (g1, _) = resolve_entry_with_index(&index, &a).expect("v3 disk hit");
        let index2 = build_multi_entry_index(&roots);
        let (g2, _) = resolve_entry_with_index(&index2, &a).expect("v3 disk repeat");
        assert!(
            std::rc::Rc::ptr_eq(&g1, &g2),
            "v3 disk hit must install into share"
        );
        assert_eq!(
            decode_count(),
            decodes_before + 1,
            "first v3 disk hit decodes once; repeat must not"
        );
    });

    // Arm 3 — legacy v1 on disk: LegacyFormatMiss -> cold compute, no provider probe.
    let legacy_dir = dir.join("legacy-cache");
    fs::create_dir_all(&legacy_dir).expect("legacy cache dir");
    write_raw_artifact_for_test(
        &legacy_dir,
        &subject,
        &build_legacy_v1_probe_artifact(&subject),
    )
    .expect("legacy write");
    match probe(&legacy_dir, &subject) {
        CacheProbeResult::LegacyMigrationRequired { format_version } => {
            assert_eq!(format_version, 1, "legacy v1 must not expose v3 parts");
        }
        other => panic!("legacy v1 must probe as LegacyMigrationRequired: {other:?}"),
    }
    with_cache_env(&legacy_dir, || {
        let decodes_before = decode_count();
        let index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&index, &a).expect("legacy must cold-rebuild");
        assert_eq!(
            decode_count(),
            decodes_before,
            "legacy format must not decode incomplete v1 artifact"
        );
    });

    // Arm 4 — semantically incomplete v3 on disk: typed refusal through live resolve.
    let incomplete_semantic = resolved_graph_parts_semantic_digest(
        &encoded.graph_digest,
        encoded.graph_bytes.len() as u64,
        &encoded.indices_digest,
        encoded.indices_bytes.len() as u64,
        UNION_PART_ABSENT_DIGEST,
        0,
    )
    .expect("incomplete semantic");
    let incomplete_bytes = build_incomplete_v3_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &incomplete_semantic,
    )
    .expect("incomplete bytes");
    let incomplete_dir = dir.join("incomplete-cache");
    fs::create_dir_all(&incomplete_dir).expect("incomplete cache dir");
    write_raw_artifact_for_test(&incomplete_dir, &subject, &incomplete_bytes)
        .expect("incomplete write");
    match lookup(&incomplete_dir, &subject) {
        CacheLookupResult::RejectedHit(CacheRejectReason::ContentDigestMismatch) => {}
        other => panic!("incomplete v3 lookup must refuse, not widen to hit: {other:?}"),
    }
    with_cache_env(&incomplete_dir, || {
        let decodes_before = decode_count();
        let index = build_multi_entry_index(&roots);
        let err = resolve_entry_with_index(&index, &a).expect_err("incomplete v3 must refuse");
        assert!(
            err.contains("incomplete") && err.contains("compile_clean_diagnostic_union"),
            "typed incomplete refusal must name missing output: {err}"
        );
        assert_eq!(
            decode_count(),
            decodes_before,
            "incomplete v3 must not decode or cold-rebuild"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

/// review 46678: legacy v1 must not block v3 materialization — cold rebuild replaces
/// the legacy row and a fresh index gets a verified v3 disk hit.
#[test]
fn legacy_v1_cold_rebuild_migrates_to_v3_on_fresh_index() {
    let dir = temp_dir("legacy-migrate");
    let (roots, a, _, _) = write_fixture(&dir);
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let legacy_dir = dir.join("legacy-migrate-cache");
    fs::create_dir_all(&legacy_dir).expect("legacy cache dir");
    write_raw_artifact_for_test(
        &legacy_dir,
        &subject,
        &build_legacy_v1_probe_artifact(&subject),
    )
    .expect("legacy write");

    with_cache_env(&legacy_dir, || {
        let index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&index, &a).expect("legacy cold rebuild must migrate");
    });
    match probe(&legacy_dir, &subject) {
        CacheProbeResult::Hit(_hit) => {
            // cold rebuild replaced legacy v1 with v3 on disk
        }
        other => panic!("expected v3 probe after migration: {other:?}"),
    }

    with_cache_env(&legacy_dir, || {
        let decodes_before = decode_count();
        let index2 = build_multi_entry_index(&roots);
        resolve_entry_with_index(&index2, &a).expect("fresh index v3 disk hit");
        assert_eq!(
            decode_count(),
            decodes_before + 1,
            "fresh index must decode v3 once after legacy migration"
        );
    });

    let _ = fs::remove_dir_all(&dir);
}

/// review 46697: commit revalidates disposition — a verified v3 row is never deleted
/// because an earlier classify saw legacy.
#[test]
fn v3_write_refuses_when_verified_artifact_already_present() {
    let dir = temp_dir("v3-write-refuse");
    let (roots, a, _, _) = write_fixture(&dir);
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve");
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let v3_bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("v3 bytes");
    let cache_dir = dir.join("v3-present");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    write_raw_artifact_for_test(&cache_dir, &subject, &v3_bytes).expect("seed v3");
    let artifact_path = cache_dir.join(&subject[..2]).join(format!("{subject}.bin"));

    let outcome = write_resolved_graph_cache(
        &cache_dir,
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("write against existing v3");
    assert_eq!(
        outcome,
        CacheWriteOutcome::AlreadyExists,
        "verified v3 must refuse overwrite"
    );
    let on_disk = fs::read(&artifact_path).expect("read v3 artifact");
    assert_eq!(
        on_disk, v3_bytes,
        "write must not replace verified v3 bytes"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// review 46613: provider-approved probe must bind the reopened artifact header before
/// decode — a replacement file that is internally valid but header-mismatched refuses.
#[test]
fn verified_lookup_refuses_artifact_replaced_after_probe() {
    let dir = temp_dir("verified-probe-toctou");
    let (roots, a, _, _) = write_fixture(&dir);
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let subject = subject_digest_for_closure(&sources);
    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve fixture");
    let (request_key, semantic) = provider_keys_for_graph(
        &roots,
        &a,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
    );
    let correct_bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &semantic,
    )
    .expect("correct bytes");
    let wrong_semantic = resolved_graph_parts_semantic_digest(
        &encode_resolved_graph_parts(&graph, si.as_ref(), &empty_compile_clean_diags())
            .expect("encode")
            .graph_digest,
        1,
        &encode_resolved_graph_parts(&graph, si.as_ref(), &empty_compile_clean_diags())
            .expect("encode")
            .indices_digest,
        1,
        UNION_PART_ABSENT_DIGEST,
        0,
    )
    .expect("wrong semantic");
    assert_ne!(
        semantic, wrong_semantic,
        "test needs distinct semantic digests"
    );
    let swapped_bytes = build_valid_artifact_bytes(
        &subject,
        &graph,
        si.as_ref(),
        &empty_compile_clean_diags(),
        &request_key,
        &wrong_semantic,
    )
    .expect("swapped bytes");

    let cache_dir = dir.join("toctou-cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    write_raw_artifact_for_test(&cache_dir, &subject, &correct_bytes).expect("write correct");
    let approved = match probe(&cache_dir, &subject) {
        CacheProbeResult::Hit(hit) => hit,
        other => panic!("expected probe hit: {other:?}"),
    };
    write_raw_artifact_for_test(&cache_dir, &subject, &swapped_bytes).expect("swap artifact");
    match lookup_verified_probe(&cache_dir, &subject, &approved) {
        CacheLookupResult::RejectedHit(_) => {}
        other => panic!("verified lookup must refuse header mismatch after swap: {other:?}"),
    }
    match lookup(&cache_dir, &subject) {
        CacheLookupResult::Hit(_) => {}
        other => panic!("unverified lookup still decodes swapped artifact: {other:?}"),
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
// serves a fresh index's first touch of a subject and installs into the share
// once the provider can serve a complete v2 artifact — no cold-resolve widen.
#[test]
fn same_subject_resolves_share_one_graph_store_hits_v2_disk() {
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
        let (g3, _) = resolve_entry_with_index(&index2, &a).expect("v2 store hit must install");
        let decodes_after_disk = decode_count();
        assert_eq!(
            decodes_after_disk,
            decodes_before + 1,
            "first disk touch decodes once"
        );
        let (g4, _) = resolve_entry_with_index(&index2, &a).expect("memo after disk install");
        assert!(
            std::rc::Rc::ptr_eq(&g3, &g4),
            "disk hit must install into share; repeat must serve by reference"
        );
        assert_eq!(
            decode_count(),
            decodes_after_disk,
            "memo repeat must not decode again"
        );
    });
}

// Repeat-resolve through the disk seam must not re-enter the materialization provider.
//
// Root cause (2026-08-03): the seam suppressed provider routing on the PROBE direction
// only. The STORE direction ignored the same flag, so writing an artifact during the
// provider's own bootstrap called back into `materialization_provider_ctx` while its
// memo slot was still empty — rebuilding the provider closure, whose nested resolve
// re-entered the store, unbounded, at roughly 1GiB of resolved graph per level. The
// observed shape was an OOM kill, not a diagnostic.
//
// Discriminating on the counted bootstrap window: a resolve that stores while the
// provider is booting MUST record a skip and MUST leave the provider ctx built exactly
// once. Before the fix this test does not merely fail — it exhausts memory — which is
// precisely why the wall below exists as a separately loud control.
#[test]
fn repeat_resolve_through_disk_seam_does_not_reenter_provider() {
    let dir = temp_dir("no-reenter");
    let (roots, a, _b, _c) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    with_cache_env(&cache_dir, || {
        let skips_before = provider_bootstrap_store_skip_count();

        let index = build_multi_entry_index(&roots);
        let (_g1, _) = resolve_entry_with_index(&index, &a).expect("cold resolve");
        let builds_after_cold = materialization_provider_ctx_build_count_for_test();
        assert!(
            builds_after_cold >= 1,
            "the store path must have booted the provider ctx at least once"
        );
        assert!(
            provider_bootstrap_store_skip_count() > skips_before,
            "the provider's own bootstrap resolve must record a counted store skip,              never silently recurse into the provider"
        );

        // A fresh index for the same subject: the disk-hit direction.
        let index2 = build_multi_entry_index(&roots);
        let (_g3, _) = resolve_entry_with_index(&index2, &a).expect("disk hit");
        assert_eq!(
            materialization_provider_ctx_build_count_for_test(),
            builds_after_cold,
            "a repeat resolve through the disk seam must reuse the provider ctx,              never rebuild it (a rebuild is the unbounded-recursion shape)"
        );
    });
}

// The construction wall itself, exercised directly and loudly: a demand for the
// provider raised while the provider's own authority closure is still resolving is a
// typed, located refusal. Remove the wall and this rebuilds instead of refusing.
#[test]
fn reentrant_provider_ctx_construction_refuses() {
    let err = provider_ctx_reentrancy_refusal_for_test()
        .err()
        .expect("re-entrant provider-ctx construction must refuse, not rebuild");
    assert!(
        err.contains("re-entrant provider-ctx construction refused"),
        "refusal must be located and name the class, got: {err}"
    );
}

// `cross_process_hit_skips_semantic_recompute` was written and executed against
// `same_subject_resolves_share_one_graph_store_hits_v2_disk`'s shape, then removed
// rather than landed, because at the time it reliably OOM-killed the runner instead
// of failing loud and located. That finding is ROOT-CAUSED AND FIXED (#7728) — the
// seam's store direction ignored the provider-bootstrap suppression flag that its
// probe direction honoured, so the provider's own bootstrap resolve rebuilt the
// provider closure recursively. The two tests above are the controls that replaced it
// at the reentrancy layer; this is the remaining warm-hit skip-proof itself, enrolled
// (DESIGN.md "disk-tier repeat-resolve memory growth" open thread).
//
// The discriminating oracle is TYPECHECK_COMPUTE_COUNT (the same once-per-node counter
// `union_resolve_receipts_test.rs` uses for the in-process share tier): a disk-tier hit
// for an already-materialized subject must return via
// `install_cross_process_materialization_hit` without ever entering the per-module
// typecheck loop that calls `bump_typecheck_compute_count`. A regression that silently
// falls back to a full semantic recompute on a "hit" would still return a correct graph
// — only the counter would move — which is exactly the failure mode a byte-comparison
// of the result cannot catch.
#[test]
fn cross_process_hit_skips_semantic_recompute() {
    let dir = temp_dir("skip-recompute");
    let (roots, a, _b, _c) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    with_cache_env(&cache_dir, || {
        with_typecheck_compute_count_receipt(|| {
            // Cold: first touch of the subject in this process, populates both the
            // in-process share and (once the provider can serve a complete artifact)
            // the disk-tier store.
            let index = build_multi_entry_index(&roots);
            reset_typecheck_compute_count();
            resolve_entry_with_index(&index, &a).expect("cold resolve populates the store");
            let cold = typecheck_compute_count();
            assert!(
                cold > 0,
                "the cold resolve must genuinely compute something, or a flat 0 downstream \
                 proves nothing about skipping recompute"
            );

            // A fresh index for the same subject — the shape of a new process's first
            // touch, served from the disk-tier store rather than the in-process share.
            let index2 = build_multi_entry_index(&roots);
            reset_typecheck_compute_count();
            resolve_entry_with_index(&index2, &a).expect("disk-tier hit");
            assert_eq!(
                typecheck_compute_count(),
                0,
                "a disk-tier hit for an already-materialized subject must not re-run semantic \
                 typecheck — TYPECHECK_COMPUTE_COUNT staying at 0 is the proof that the warm \
                 path skips recompute rather than silently rebuilding it"
            );
        });
    });
}
