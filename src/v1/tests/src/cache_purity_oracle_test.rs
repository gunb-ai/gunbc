//! Executable warm==cold cache-purity detective (DESIGN §5; ROADMAP §2 P1).
//!
//! Authority: `extdeps/realization/cache_purity.dag` (the verdict carrier) + the v1 handler
//! `cache_purity_oracle.rs`. These are the DISCRIMINATING witnesses the §5 spec-without-execution
//! trap demands: a REAL consumer green-by-execution PLUS an injected hidden input that goes RED.
//!
//!   1. `real_resolved_graph_cache_round_trips_byte_identical` — exercises the REAL kernel
//!      (`resolved_graph_cache::{lookup,write}`): a COLD resolve (forced miss → compute → write)
//!      then a WARM resolve (cache hit) must be byte-identical after canonicalization. Green by
//!      running the actual cache path.
//!   2. `real_resolved_graph_realization_is_pure_under_nonkeyed_probes` — runs the generic oracle
//!      over the REAL resolve realization with probes for inputs the key legitimately ignores (an
//!      unrelated env var, an unrelated sibling file). The real kernel is PURE w.r.t. them → green,
//!      proving the oracle does not false-positive on the correct kernel.
//!   3. `oracle_raises_loud_located_error_on_injected_impurity` — the RED falsifier: a realization
//!      with a HIDDEN non-keyed input (read at realize time, absent from the content-key). The
//!      oracle MUST raise a located, typed, loud `CachePurityViolation` naming the axis. Proves
//!      the detective has teeth — without it, (1)/(2) would pass vacuously.
//!   4. `oracle_skips_probes_that_move_the_content_key` — a probe that moves the key is a DECLARED
//!      axis (a miss, not a stale hit); the oracle must SKIP it, not report impurity.

use std::cell::Cell;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cache_purity_oracle::{
    audit_warm_equals_cold, AuditedRealization, CachePurityViolation, HiddenInputProbe,
};
use v1_compiler::cli_run::{
    build_multi_entry_index, load_sources_for_entry, resolve_entry_graph, resolve_entry_with_index,
};
use v1_compiler::resolved_graph_cache::{
    serialize_fixture_payload_for_test, subject_digest_for_closure,
};
use v1_compiler::v1_compiler_compile::ResolvedGraph;
use v1_compiler::v1_rt::{self, Hash};
use v1_compiler::v1_std_core::NewlineIndex;

// The cache dir is a process-global env var; serialize the tests that touch it (libtest runs
// `#[test]` in parallel). Mirrors resolve_cross_process_cache_test.rs's CACHE_ENV_MUTEX.
static CACHE_ENV_MUTEX: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "gunbc-cache-purity-{label}-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Minimal resolvable fixture: an entry importing one shared module. Returns (roots, entry).
fn write_fixture(dir: &std::path::Path) -> (Vec<String>, String) {
    let shared = "module test.shared\nfn val() -> Int { 7 }\n";
    let entry = "module test.entry\n\
        import test.shared { val }\n\
        fn witness_true() -> Bool { val() == 7 }\n";
    fs::write(dir.join("shared.dag"), shared).expect("write shared");
    fs::write(dir.join("entry.dag"), entry).expect("write entry");
    let roots = vec![dir.to_string_lossy().into_owned()];
    let entry = dir.join("entry.dag").to_string_lossy().into_owned();
    (roots, entry)
}

/// Canonical serialized bytes of a resolved graph. Round-trips through `serde_json::Value`
/// (whose object keys are a sorted `BTreeMap` — `preserve_order` is off in this workspace) so
/// `source_indices`'s `HashMap` iteration order cannot make a byte compare false-fail. This is
/// exactly the canonicalization the CRIT-1 boundary in resolve_cross_process_cache_test.rs flags
/// as the prerequisite for a real byte-identity oracle.
fn canonical_graph_bytes(
    graph: &ResolvedGraph,
    source_indices: &std::collections::HashMap<String, Rc<NewlineIndex>>,
) -> Vec<u8> {
    let raw = serialize_fixture_payload_for_test(graph, source_indices).expect("serialize payload");
    let value: serde_json::Value = serde_json::from_slice(&raw).expect("payload is valid json");
    serde_json::to_vec(&value).expect("re-serialize canonical")
}

struct CacheEnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
}

impl CacheEnvGuard {
    fn set(cache_dir: &std::path::Path) -> Self {
        let lock = CACHE_ENV_MUTEX.lock().expect("cache env mutex");
        let prev = std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR");
        std::env::set_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR", cache_dir);
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

// === 1. REAL kernel: a warm hit is byte-identical to the cold compute it cached ===================

#[test]
fn real_resolved_graph_cache_round_trips_byte_identical() {
    let dir = temp_dir("roundtrip");
    let (roots, entry) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let _guard = CacheEnvGuard::set(&cache_dir);

    // COLD: fresh index, empty cache → forced miss → compute → write.
    let cold_index = build_multi_entry_index(&roots);
    let (cold_graph, cold_si) =
        resolve_entry_with_index(&cold_index, &entry).expect("cold resolve");
    let cold_bytes = canonical_graph_bytes(&cold_graph, &cold_si);

    // WARM: a fresh index over the same sources → cache HIT, served from the on-disk artifact.
    let warm_index = build_multi_entry_index(&roots);
    let (warm_graph, warm_si) =
        resolve_entry_with_index(&warm_index, &entry).expect("warm resolve");
    let warm_bytes = canonical_graph_bytes(&warm_graph, &warm_si);

    assert_eq!(
        v1_rt::bytes_identity_hash(&warm_bytes),
        v1_rt::bytes_identity_hash(&cold_bytes),
        "warm cache hit must be byte-identical to the cold compute it cached (DESIGN §5 warm==cold)"
    );

    let _ = fs::remove_dir_all(&dir);
}

// === Real-kernel adapter for the generic oracle ===================================================

/// The real resolved-graph realization, audited by the generic oracle. `content_key` is the
/// production cache key (`subject_digest_for_closure`); `realize_cold` is a fresh, uncached
/// resolve serialized canonically.
struct ResolvedGraphRealization {
    roots: Vec<String>,
    entry: String,
}

impl AuditedRealization for ResolvedGraphRealization {
    fn content_key(&self) -> Hash {
        let sources = load_sources_for_entry(&self.roots, &self.entry).expect("sources");
        subject_digest_for_closure(&sources)
    }

    fn realize_cold(&self) -> Vec<u8> {
        let (graph, si) = resolve_entry_graph(&self.roots, &self.entry).expect("cold resolve");
        canonical_graph_bytes(&graph, &si)
    }
}

// === 2. REAL realization is PURE under inputs the key legitimately ignores (no false positive) ====

#[test]
fn real_resolved_graph_realization_is_pure_under_nonkeyed_probes() {
    let _lock = CACHE_ENV_MUTEX.lock().expect("cache env mutex"); // serialize env touch below
    let dir = temp_dir("pure");
    let (roots, entry) = write_fixture(&dir);
    let sibling = dir.join("unrelated_not_imported.dag");

    let realization = ResolvedGraphRealization {
        roots: roots.clone(),
        entry: entry.clone(),
    };

    // Probe A — an unrelated env var the resolver never reads.
    let env_key = "GUNBC_CACHE_PURITY_PROBE_UNRELATED";
    // Probe B — a sibling file NOT in the entry's import closure (so NOT in the closure digest).
    let sibling_for_perturb = sibling.clone();
    let sibling_for_restore = sibling.clone();

    let mut probes = [
        HiddenInputProbe {
            axis: "unrelated_env_var",
            perturb: Box::new(|| std::env::set_var(env_key, "perturbed")),
            restore: Box::new(|| std::env::remove_var(env_key)),
        },
        HiddenInputProbe {
            axis: "unrelated_sibling_file",
            perturb: Box::new(move || {
                fs::write(
                    &sibling_for_perturb,
                    "module test.unrelated\nfn z() -> Int { 0 }\n",
                )
                .expect("write sibling");
            }),
            restore: Box::new(move || {
                let _ = fs::remove_file(&sibling_for_restore);
            }),
        },
    ];

    let verdict = audit_warm_equals_cold(&realization, &mut probes);
    assert!(
        verdict.is_ok(),
        "the real resolved-graph realization must be PURE w.r.t. inputs its key ignores; \
         the oracle must not false-positive: {verdict:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

// === 3. RED falsifier: an injected hidden non-keyed input makes warm!=cold → loud located error ===

/// A realization with an INJECTED IMPURITY: `realize_cold` mixes in a hidden byte read from a
/// `Cell` that `content_key` does NOT fold. This is the §5 disease the detective must catch — an
/// input read at realize time but absent from the key, so a warm hit serves a stale result.
struct ImpureRealization {
    fixed_key: Hash,
    hidden_input: Rc<Cell<u8>>,
}

impl AuditedRealization for ImpureRealization {
    fn content_key(&self) -> Hash {
        // The key is FIXED — it does NOT depend on `hidden_input` (the bug).
        self.fixed_key.clone()
    }

    fn realize_cold(&self) -> Vec<u8> {
        // The output DOES depend on the hidden input — read but never keyed.
        vec![0xAB, 0xCD, self.hidden_input.get()]
    }
}

#[test]
fn oracle_raises_loud_located_error_on_injected_impurity() {
    let hidden = Rc::new(Cell::new(0u8));
    let realization = ImpureRealization {
        fixed_key: "feedfacefeedface".to_string(),
        hidden_input: hidden.clone(),
    };

    let hidden_for_perturb = hidden.clone();
    let hidden_for_restore = hidden.clone();
    let mut probes = [HiddenInputProbe {
        axis: "injected_hidden_counter",
        // Move the hidden input WITHOUT touching the (fixed) content-key.
        perturb: Box::new(move || hidden_for_perturb.set(0xFF)),
        restore: Box::new(move || hidden_for_restore.set(0x00)),
    }];

    let result = audit_warm_equals_cold(&realization, &mut probes);
    let violation: CachePurityViolation =
        result.expect_err("an input read at realize time but absent from the key MUST be caught");

    // Located: names the un-keyed axis.
    assert_eq!(
        violation.unkeyed_axis, "injected_hidden_counter",
        "the violation must LOCATE the read-but-unkeyed axis"
    );
    // Typed + true divergence: the key stayed fixed while warm != cold.
    assert_eq!(violation.content_key, "feedfacefeedface");
    assert_ne!(
        violation.warm_digest, violation.cold_digest,
        "warm (cached baseline) must differ from cold (fresh recompute) — that IS the impurity"
    );
    // Loud: the Display is a fail-closed §5 error, not a warning, and names the axis.
    let shouted = format!("{violation}");
    assert!(
        shouted.contains("CACHE PURITY VIOLATION") && shouted.contains("injected_hidden_counter"),
        "the error must be LOUD and name the axis; got: {shouted}"
    );
}

// === 4. A probe that moves the content-key is a DECLARED axis — skipped, not flagged =============

/// A faithfully content-keyed realization: the key AND the output both move with the input, so a
/// change is a cache MISS, never a stale hit. The oracle must SKIP such a probe.
struct KeyedRealization {
    input: Rc<Cell<u8>>,
}

impl AuditedRealization for KeyedRealization {
    fn content_key(&self) -> Hash {
        v1_rt::atom_identity_hash(format!("keyed-{}", self.input.get()))
    }
    fn realize_cold(&self) -> Vec<u8> {
        vec![self.input.get()]
    }
}

#[test]
fn oracle_skips_probes_that_move_the_content_key() {
    let input = Rc::new(Cell::new(1u8));
    let realization = KeyedRealization {
        input: input.clone(),
    };
    let input_for_perturb = input.clone();
    let input_for_restore = input.clone();
    let mut probes = [HiddenInputProbe {
        axis: "declared_keyed_input",
        perturb: Box::new(move || input_for_perturb.set(2)),
        restore: Box::new(move || input_for_restore.set(1)),
    }];

    // The probe moves BOTH the key and the output → it is a declared axis (a miss), not impurity.
    assert!(
        audit_warm_equals_cold(&realization, &mut probes).is_ok(),
        "a probe that moves the content-key is a DECLARED axis (a miss, not a stale hit) — skip it"
    );
}
