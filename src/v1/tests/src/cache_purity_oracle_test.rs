use std::cell::Cell;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{
    build_multi_entry_index, load_sources_for_entry, resolve_entry_graph, resolve_entry_with_index,
};
use v1_compiler::resolved_graph_cache::{
    audit_warm_equals_cold, AuditedRealization, CachePurityViolation, HiddenInputProbe,
};
use v1_compiler::resolved_graph_cache::{
    serialize_fixture_payload_for_test, subject_digest_for_closure,
};
use v1_compiler::v1_compiler_compile::ResolvedGraph;
use v1_compiler::v1_rt::{self, Hash};
use v1_compiler::v1_std_core::NewlineIndex;

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

fn sort_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                out.insert(
                    key.clone(),
                    sort_json_value(map.get(&key).expect("key").clone()),
                );
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sort_json_value).collect())
        }
        other => other,
    }
}

fn json_diff_path(a: &serde_json::Value, b: &serde_json::Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (serde_json::Value::Object(ma), serde_json::Value::Object(mb)) => {
            let mut keys: Vec<&String> = ma.keys().chain(mb.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                let next = format!("{path}.{key}");
                let av = ma.get(key).unwrap_or(&serde_json::Value::Null);
                let bv = mb.get(key).unwrap_or(&serde_json::Value::Null);
                if let Some(found) = json_diff_path(av, bv, &next) {
                    return Some(found);
                }
            }
            Some(path.to_string())
        }
        (serde_json::Value::Array(aa), serde_json::Value::Array(bb)) => {
            if aa.len() != bb.len() {
                return Some(format!("{path}[len {} vs {}]", aa.len(), bb.len()));
            }
            for (i, (av, bv)) in aa.iter().zip(bb.iter()).enumerate() {
                if let Some(found) = json_diff_path(av, bv, &format!("{path}[{i}]")) {
                    return Some(found);
                }
            }
            Some(path.to_string())
        }
        _ => Some(path.to_string()),
    }
}

fn canonical_graph_bytes(
    graph: &ResolvedGraph,
    source_indices: &std::collections::HashMap<String, Rc<NewlineIndex>>,
) -> Vec<u8> {
    serialize_fixture_payload_for_test(graph, source_indices).expect("serialize payload")
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

#[test]
fn real_resolved_graph_cache_round_trips_byte_identical() {
    let dir = temp_dir("roundtrip");
    let (roots, entry) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let _guard = CacheEnvGuard::set(&cache_dir);

    let cold_index = build_multi_entry_index(&roots);
    let (cold_graph, cold_si) =
        resolve_entry_with_index(&cold_index, &entry).expect("cold resolve");
    let cold_bytes = canonical_graph_bytes(&cold_graph, &cold_si);

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

#[test]
fn resolved_graph_realization_is_stable_back_to_back() {
    let dir = temp_dir("stable");
    let (roots, entry) = write_fixture(&dir);
    let realization = ResolvedGraphRealization {
        roots: roots.clone(),
        entry: entry.clone(),
    };
    let bytes1 = realization.realize_cold();
    let bytes2 = realization.realize_cold();
    if bytes1 != bytes2 {
        let v1: serde_json::Value = serde_json::from_slice(&bytes1).expect("json1");
        let v2: serde_json::Value = serde_json::from_slice(&bytes2).expect("json2");
        let s1 = sort_json_value(v1.clone());
        let s2 = sort_json_value(v2.clone());
        if s1 != s2 {
            let keys = ["graph", "source_indices"];
            for key in keys {
                let h1 = v1_rt::bytes_identity_hash(
                    &serde_json::to_vec(&s1.get(key).unwrap_or(&serde_json::Value::Null)).unwrap(),
                );
                let h2 = v1_rt::bytes_identity_hash(
                    &serde_json::to_vec(&s2.get(key).unwrap_or(&serde_json::Value::Null)).unwrap(),
                );
                if h1 != h2 {
                    let ga = s1.get(key).expect("graph key");
                    let gb = s2.get(key).expect("graph key");
                    let diff = json_diff_path(ga, gb, key).unwrap_or_else(|| key.to_string());
                    panic!(
                        "back-to-back compile diverged on payload.{key} at {diff}: {h1} vs {h2}"
                    );
                }
            }
            panic!(
                "back-to-back compile diverged outside graph/source_indices after canonical sort"
            );
        }
        panic!(
            "back-to-back compile diverged only in raw serde key order (canonical sort matched)"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn real_resolved_graph_realization_is_pure_under_nonkeyed_probes() {
    let dir = temp_dir("pure");
    let (roots, entry) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    let _guard = CacheEnvGuard::set(&cache_dir);
    let sibling = dir.join("unrelated_not_imported.dag");

    let realization = ResolvedGraphRealization {
        roots: roots.clone(),
        entry: entry.clone(),
    };

    let env_key = "GUNBC_CACHE_PURITY_PROBE_UNRELATED";
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

struct ImpureRealization {
    fixed_key: Hash,
    hidden_input: Rc<Cell<u8>>,
}

impl AuditedRealization for ImpureRealization {
    fn content_key(&self) -> Hash {
        self.fixed_key.clone()
    }

    fn realize_cold(&self) -> Vec<u8> {
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
        perturb: Box::new(move || hidden_for_perturb.set(0xFF)),
        restore: Box::new(move || hidden_for_restore.set(0x00)),
    }];

    let result = audit_warm_equals_cold(&realization, &mut probes);
    let violation: CachePurityViolation =
        result.expect_err("an input read at realize time but absent from the key MUST be caught");

    assert_eq!(
        violation.unkeyed_axis, "injected_hidden_counter",
        "the violation must LOCATE the read-but-unkeyed axis"
    );
    assert_eq!(violation.content_key, "feedfacefeedface");
    assert_ne!(
        violation.warm_digest, violation.cold_digest,
        "warm (cached baseline) must differ from cold (fresh recompute) — that IS the impurity"
    );
    let shouted = format!("{violation}");
    assert!(
        shouted.contains("CACHE PURITY VIOLATION") && shouted.contains("injected_hidden_counter"),
        "the error must be LOUD and name the axis; got: {shouted}"
    );
}

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

    assert!(
        audit_warm_equals_cold(&realization, &mut probes).is_ok(),
        "a probe that moves the content-key is a DECLARED axis (a miss, not a stale hit) — skip it"
    );
}
