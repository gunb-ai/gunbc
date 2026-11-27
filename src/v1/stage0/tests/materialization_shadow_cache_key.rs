//! S1/S4 cache-key fixture for the v1 compiler reuse experiment.
//!
//! This is NOT the production `build_newline_index` observer. The newline index is a
//! content-only answer, so it can never show a provider-dependent disagreement. This fixture
//! keys the answer to `v1.symbol.index::build_symbol_index_census` — a resolution-dependent
//! front-end value — with two candidate cache keys, and reuses the observer's store/record
//! machinery to observe the difference:
//!
//!   * S1 bytes-only key — derived from the compiler image + the consumer's logical path +
//!     the consumer's bytes alone. It does not see any other module in the tree.
//!   * S4 corrected key — derived from the compiler image + the consumer + a SORTED,
//!     source-derived dependency set over the whole closure. Adding a previously absent
//!     provider changes that set, and therefore the key. The set is iterated in sorted order
//!     (never `HashMap`/`HashSet` iteration order), so the same tree yields the same key in
//!     separate processes.
//!
//! Scenario ids E–H (defined in README.md) run in fresh OS processes: the test binary is
//! re-executed once per step via `--exact scenario_worker`. No stored answer is ever fed
//! back to the compiler; the compiler recomputes fresh in every process.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use v1_compiler::closure_identity::{
    shadow_compare_and_record, shadow_read_prior, shadow_record, ShadowFingerprint as Fingerprint,
    ShadowObservation as Observation,
};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::build_symbol_index_census;
use v1_compiler::v1_compiler_normalize::normalize_graph;
use v1_compiler::v1_std_core::{is_error_diagnostic, CompilerDiagnostic, LeafOwner};

const QUESTION: &str = "v1.symbol.index::build_symbol_index_census";
const SCHEMA: &[u8] = b"symbol-census-cache-key-v1";
const PREFIX: &str = "[cache-key-fixture] ";
const WORKER_ENV: &str = "GBC_CK_WORKER";
const TREE_ENV: &str = "GBC_CK_TREE";
const STORE_ENV: &str = "GBC_CK_STORE";
const DERIV_ENV: &str = "GBC_CK_DERIV";
const CONSUMER_ENV: &str = "GBC_CK_CONSUMER";

// A consumer file whose own bytes are held fixed across the provider scenarios. The bare
// leaf `Banner` is its declared reference; a separate provider declaring the same leaf flips
// the census's global bare-name resolution from a single owner to an ambiguous binding.
const CONSUMER: &str = "helper.dag";
fn consumer_content(literal: i32) -> String {
    format!(
        "module census.helper\nfn answer() -> Int {{ {literal} }}\ntype Banner {{ rank: Int }}\n"
    )
}
const PROVIDER: &str = "other.dag";
const PROVIDER_CONTENT: &str = "module census.other\ntype Banner { rank: Int }\n";
// A second file declaring the SAME module path as the consumer. It is the wrong provider:
// the compiler refuses a duplicate module before any census is computed.
const DUPLICATE: &str = "dupe.dag";
const DUPLICATE_CONTENT: &str = "module census.helper\nfn unused() -> Int { 0 }\n";

#[derive(Debug, Serialize, Deserialize)]
struct WorkerResult {
    derivation: String,
    input_digest: String,
    output_digest: String,
    lookup: String,
    comparison: String,
    refused: bool,
    refusal_note: String,
}

// ---------------------------------------------------------------------------
// Store/record machinery (mirrors cli_run::materialization_shadow exactly).
// The observer's items are private to that module, so the fixture carries a faithful copy
// of the same record format and comparison semantics rather than reaching into the seed.
// ---------------------------------------------------------------------------

fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

// ---------------------------------------------------------------------------
// Answer and key derivation.
// ---------------------------------------------------------------------------

fn compiler_image() -> Fingerprint {
    // The image identity comes from the seed's single compiler-identity authority
    // (`closure_identity::transform_content_digest`), not a second `/proc/self/exe` read in
    // the test — that path would go stale once a cargo build re-links the binary.
    let mut hash = Sha256::new();
    hash.update(v1_compiler::closure_identity::transform_content_digest().as_bytes());
    hash.finalize().into()
}

/// A deterministic canonical byte form of a `serde_json::Value`: object keys are sorted by
/// hand, so the digest of the census answer does not depend on any hash-map iteration order.
fn canonical(value: &serde_json::Value, out: &mut Vec<u8>) {
    match value {
        serde_json::Value::Null => out.push(b'0'),
        serde_json::Value::Bool(b) => out.push(if *b { b'1' } else { b'2' }),
        serde_json::Value::Number(n) => {
            out.push(b'#');
            out.extend(n.to_string().as_bytes());
        }
        serde_json::Value::String(s) => {
            out.push(b's');
            field_len(out, s.as_bytes());
        }
        serde_json::Value::Array(items) => {
            out.push(b'[');
            for item in items {
                canonical(item, out);
            }
            out.push(b']');
        }
        serde_json::Value::Object(map) => {
            out.push(b'{');
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                field_len(out, key.as_bytes());
                canonical(&map[key], out);
            }
            out.push(b'}');
        }
    }
}

fn field_len(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend((bytes.len() as u64).to_le_bytes());
    out.extend(bytes);
}

fn census_answer_digest(index: &v1_compiler::v1_compiler_infer_env::SymbolIndex) -> Fingerprint {
    let value = serde_json::to_value(index).expect("census answer serializes");
    let mut bytes = Vec::new();
    canonical(&value, &mut bytes);
    let mut hash = Sha256::new();
    field(&mut hash, SCHEMA);
    field(&mut hash, QUESTION.as_bytes());
    hash.update(&bytes);
    hash.finalize().into()
}

/// S1: the bytes-only key — compiler image, the consumer's module path and the consumer bytes
/// alone. It does not see any other module in the tree, which is the defect being refuted.
fn s1_key(image: &Fingerprint, consumer_module: &str, consumer_bytes: &str) -> Fingerprint {
    let mut hash = Sha256::new();
    field(&mut hash, SCHEMA);
    field(&mut hash, QUESTION.as_bytes());
    field(&mut hash, image);
    field(&mut hash, b"s1");
    field(&mut hash, consumer_module.as_bytes());
    field(&mut hash, consumer_bytes.as_bytes());
    hash.finalize().into()
}

// ---------------------------------------------------------------------------
// Real compiler over real sources (library), in a fresh process.
// ---------------------------------------------------------------------------

struct CensusOutcome {
    duplicate: bool,
    messages: Vec<String>,
    digest: Option<Fingerprint>,
}

fn compile_census(sources: &[(String, String)]) -> CensusOutcome {
    let files: im::Vector<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let front = front_end_sources(Rc::new(files));
    let duplicate = front
        .diagnostics
        .iter()
        .any(|d| matches!(&*d.diagnostic, CompilerDiagnostic::DuplicateModule { .. }));
    let errors: Vec<String> = front
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_text(&d.diagnostic))
        .collect();
    if !errors.is_empty() || front.graph.is_none() {
        return CensusOutcome {
            duplicate,
            messages: errors,
            digest: None,
        };
    }
    let indices: Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>> = Rc::new(
        front
            .newline_indices
            .iter()
            .map(|index| (index.file.clone(), index.clone()))
            .collect(),
    );
    let normalized = normalize_graph(
        front.graph.clone().expect("graph after clean frontend"),
        indices.clone(),
    );
    let norm_errors: Vec<String> = normalized
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_text(&d.diagnostic))
        .collect();
    if !norm_errors.is_empty() {
        return CensusOutcome {
            duplicate,
            messages: norm_errors,
            digest: None,
        };
    }
    let census = build_symbol_index_census(normalized.graph.modules.clone(), indices);
    CensusOutcome {
        duplicate,
        messages: Vec::new(),
        digest: Some(census_answer_digest(&census)),
    }
}

fn diagnostic_text(diagnostic: &CompilerDiagnostic) -> String {
    format!("{diagnostic:?}")
}

// ---------------------------------------------------------------------------
// Worker role: re-executed once per scenario step in a fresh OS process.
// ---------------------------------------------------------------------------

fn read_tree(tree: &Path) -> Vec<(String, String)> {
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    for entry in fs::read_dir(tree).expect("read scenario tree") {
        let path = entry.expect("tree entry").path();
        if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            let name = path
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .into_owned();
            entries.insert(name, fs::read_to_string(&path).expect("read source"));
        }
    }
    entries.into_iter().collect()
}

fn module_path_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            return Some(rest.trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

fn decode_key(hex_str: &str) -> Fingerprint {
    let bytes = hex::decode(hex_str).expect("hex key");
    <[u8; 32]>::try_from(bytes.as_slice()).expect("32-byte key")
}

fn run_worker() {
    let tree = PathBuf::from(std::env::var(TREE_ENV).expect("tree env"));
    let store = PathBuf::from(std::env::var(STORE_ENV).expect("store env"));
    let derivation = std::env::var(DERIV_ENV).expect("derivation env");
    let consumer = std::env::var(CONSUMER_ENV).expect("consumer env");
    fs::create_dir_all(store.join("symbol-census-v1")).expect("create store");

    let sources = read_tree(&tree);
    let consumer_content = sources
        .iter()
        .find(|(path, _)| *path == consumer)
        .expect("consumer present")
        .1
        .clone();
    let consumer_module = module_path_from_content(&consumer_content).unwrap_or(consumer.clone());
    // The caller's module pool: (module path, source bytes). File paths never enter the key.
    let pool: Vec<(String, String)> = sources
        .iter()
        .map(|(_, content)| {
            (
                module_path_from_content(content).unwrap_or_default(),
                content.clone(),
            )
        })
        .collect();
    // The fixture's census is built over every module in the tree, so the closure roster is
    // exactly the pool's module paths.
    let closure_roster: Vec<String> = pool.iter().map(|(module, _)| module.clone()).collect();

    let key_hex = match derivation.as_str() {
        "s1" => {
            let image = compiler_image();
            hex::encode(s1_key(&image, &consumer_module, &consumer_content))
        }
        "s4" => v1_compiler::closure_identity::symbol_census_key(&pool, &closure_roster),
        other => panic!("unknown derivation {other}"),
    };
    let key = decode_key(&key_hex);

    let outcome = compile_census(&sources);
    let Some(digest) = outcome.digest else {
        emit(&WorkerResult {
            derivation,
            input_digest: key_hex,
            output_digest: String::new(),
            lookup: "refused".to_owned(),
            comparison: if outcome.duplicate {
                "duplicate-module".to_owned()
            } else {
                "diagnostic".to_owned()
            },
            refused: true,
            refusal_note: outcome.messages.join("; "),
        });
        return;
    };
    let path = store.join("symbol-census-v1").join(&key_hex);
    let observation = shadow_compare_and_record(
        &path,
        SCHEMA,
        &key,
        shadow_read_prior(&path, SCHEMA, &key),
        &digest,
    );
    let (lookup, comparison) = match observation {
        Observation::Published => ("miss", "absent"),
        Observation::Equal => ("hit", "equal"),
        Observation::Different(_) => ("hit", "different"),
        Observation::Corrupt => ("corrupt", "unavailable"),
        Observation::Io(_, _) => ("unavailable", "io-error"),
    };
    emit(&WorkerResult {
        derivation,
        input_digest: key_hex,
        output_digest: hex::encode(digest),
        lookup: lookup.to_owned(),
        comparison: comparison.to_owned(),
        refused: false,
        refusal_note: String::new(),
    });
}

fn emit(result: &WorkerResult) {
    println!(
        "{PREFIX}{}",
        serde_json::to_string(result).expect("serialize")
    );
}

#[test]
fn scenario_worker() {
    if std::env::var_os(WORKER_ENV).is_none() {
        return; // not acting as a worker; this run is the test harness itself
    }
    run_worker();
}

// ---------------------------------------------------------------------------
// Driver: build a scenario tree and spawn fresh worker processes.
// ---------------------------------------------------------------------------

struct Scenario {
    dir: PathBuf,
    tree: PathBuf,
    store: PathBuf,
}

fn new_scenario(name: &str) -> Scenario {
    let dir = std::env::temp_dir().join(format!(
        "cache-key-fixture-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let tree = dir.join("tree");
    let store = dir.join("store");
    fs::create_dir_all(&tree).unwrap();
    fs::create_dir_all(&store).unwrap();
    Scenario { dir, tree, store }
}

fn write_source(scenario: &Scenario, name: &str, content: &str) {
    fs::write(scenario.tree.join(name), content).unwrap();
}

fn worker(scenario: &Scenario, derivation: &str) -> WorkerResult {
    let exe = std::env::current_exe().expect("test executable");
    let output = Command::new(exe)
        .arg("--exact")
        .arg("scenario_worker")
        .arg("--nocapture")
        .env(WORKER_ENV, "1")
        .env("RUST_MIN_STACK", "16777216")
        .env(TREE_ENV, &scenario.tree)
        .env(STORE_ENV, &scenario.store)
        .env(DERIV_ENV, derivation)
        .env(CONSUMER_ENV, CONSUMER)
        .output()
        .expect("spawn fresh worker process");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let line = stdout
        .lines()
        .find(|line| line.contains(PREFIX))
        .unwrap_or_else(|| panic!("no worker result line\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    let json = line.rsplit_once(PREFIX).expect("worker marker").1.trim();
    let parsed: WorkerResult = serde_json::from_str(json).expect("parse worker result");
    eprintln!(
        "[obs] deriv={derivation} lookup={} comparison={} key={} fresh_answer={} refused={}",
        parsed.lookup, parsed.comparison, parsed.input_digest, parsed.output_digest, parsed.refused
    );
    parsed
}

fn record_path(scenario: &Scenario, input_digest: &str) -> PathBuf {
    scenario.store.join("symbol-census-v1").join(input_digest)
}

// ---------------------------------------------------------------------------
// Scenario E: bytes-only key serves a stale answer under a valid provider (the red).
// ---------------------------------------------------------------------------

#[test]
fn bytes_only_key_serves_a_stale_census_answer_under_a_valid_provider() {
    let scenario = new_scenario("E");
    write_source(&scenario, CONSUMER, &consumer_content(1));

    let cold = worker(&scenario, "s1");
    assert_eq!(
        (cold.lookup.as_str(), cold.comparison.as_str()),
        ("miss", "absent")
    );

    // Consumer bytes unchanged; add a previously absent, VALID provider module that declares
    // the same bare leaf, changing what the census's bare-name lookup resolves to.
    write_source(&scenario, PROVIDER, PROVIDER_CONTENT);

    let observed = worker(&scenario, "s1");
    assert_eq!(
        (observed.lookup.as_str(), observed.comparison.as_str()),
        ("hit", "different"),
        "bytes-only key must hit an intact record whose stored answer disagrees with fresh"
    );
    // The key did not move: it is the same key the cold miss published under.
    assert_eq!(cold.input_digest, observed.input_digest);
    // The stored record is intact (valid integrity bytes) and holds the OLD answer.
    let stored = fs::read(record_path(&scenario, &observed.input_digest)).expect("intact record");
    assert_eq!(stored.len(), 64);
    let stored_value = &stored[..32];
    assert_eq!(
        hex::encode(stored_value),
        cold.output_digest,
        "the intact record holds the cold (pre-provider) answer"
    );
    // The record is a valid, intact entry under the key: its integrity bytes re-verify.
    let key_bytes =
        <[u8; 32]>::try_from(hex::decode(&observed.input_digest).unwrap().as_slice()).unwrap();
    let mut stored_arr = [0u8; 32];
    stored_arr.copy_from_slice(stored_value);
    assert_eq!(
        &shadow_record(SCHEMA, &key_bytes, &stored_arr)[..],
        &stored[..],
        "the stored record is intact (valid integrity bytes)"
    );
    // The fresh answer differs from the stored one, so a naive reuse would be wrong.
    assert_ne!(observed.output_digest, cold.output_digest);

    // The provider really is resolution-dependent: confirm the census ambiguity directly so the
    // disagreement is attributable to the provider, not to nondeterminism.
    assert!(provider_makes_leaf_ambiguous(&scenario));

    fs::remove_dir_all(&scenario.dir).unwrap();
}

fn provider_makes_leaf_ambiguous(scenario: &Scenario) -> bool {
    let sources = read_tree(&scenario.tree);
    let single: Vec<(String, String)> = sources
        .iter()
        .filter(|(path, _)| path == CONSUMER)
        .cloned()
        .collect();
    let single_owner = compile_sources_owner(&single);
    let with_owner = compile_sources_owner(&sources);
    matches!(single_owner, LeafOwner::SingleOwner { .. })
        && matches!(with_owner, LeafOwner::LeafAmbiguous)
}

fn compile_sources_owner(sources: &[(String, String)]) -> LeafOwner {
    let files: im::Vector<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let compiled = compile_to_resolved(Rc::new(files));
    let graph = compiled.graph.as_ref().expect("resolved graph");
    graph
        .item_leaf_owner_modules
        .get("Banner")
        .expect("Banner declaration")
        .as_ref()
        .clone()
}

// ---------------------------------------------------------------------------
// Scenario F: corrected sorted key notices the provider, recomputes, warm repeat agrees.
// ---------------------------------------------------------------------------

#[test]
fn sorted_source_derived_key_notices_the_provider_and_agrees_on_a_warm_repeat() {
    let scenario = new_scenario("F");
    write_source(&scenario, CONSUMER, &consumer_content(1));

    let cold = worker(&scenario, "s4");
    assert_eq!(
        (cold.lookup.as_str(), cold.comparison.as_str()),
        ("miss", "absent")
    );

    write_source(&scenario, PROVIDER, PROVIDER_CONTENT);

    let first = worker(&scenario, "s4");
    // The provider changed the source-derived set, so the corrected key changed and missed.
    assert_ne!(
        first.input_digest, cold.input_digest,
        "provider must move the corrected key"
    );
    assert_eq!(
        (first.lookup.as_str(), first.comparison.as_str()),
        ("miss", "absent"),
        "the corrected key must recompute, not hit the stale answer"
    );
    // Its recomputed answer is the new, resolution-dependent one — distinct from the
    // pre-provider answer the cold step computed for this same scenario.
    assert_ne!(first.output_digest, cold.output_digest);

    // A warm repeat, in another fresh process, agrees: hit / equal, same key, same answer.
    let warm = worker(&scenario, "s4");
    assert_eq!(warm.input_digest, first.input_digest);
    assert_eq!(warm.output_digest, first.output_digest);
    assert_eq!(
        (warm.lookup.as_str(), warm.comparison.as_str()),
        ("hit", "equal"),
        "corrected key agrees on a warm repeat"
    );

    fs::remove_dir_all(&scenario.dir).unwrap();
}

// ---------------------------------------------------------------------------
// Determinism control: the same tree keyed twice in separate processes yields one key.
// ---------------------------------------------------------------------------

#[test]
fn sorted_key_is_deterministic_across_separate_processes() {
    let scenario = new_scenario("determinism");
    write_source(&scenario, CONSUMER, &consumer_content(1));
    write_source(&scenario, PROVIDER, PROVIDER_CONTENT);

    let a = worker(&scenario, "s4");
    let b = worker(&scenario, "s4");
    // Two independent processes derive the identical corrected key for the identical tree.
    assert_eq!(a.input_digest, b.input_digest);
    assert_eq!(a.output_digest, b.output_digest);
    // The second observation is a hit/equal against the first process's record.
    assert_eq!(b.lookup, "hit");
    assert_eq!(b.comparison, "equal");

    fs::remove_dir_all(&scenario.dir).unwrap();
}

// ---------------------------------------------------------------------------
// Scenario G: a body-only edit that leaves references unchanged recomputes to an equal answer.
// ---------------------------------------------------------------------------

#[test]
fn body_only_edit_recomputes_an_equal_census_answer() {
    let baseline = new_scenario("G-baseline");
    write_source(&baseline, CONSUMER, &consumer_content(1));
    let base = worker(&baseline, "s4");
    assert_eq!(
        (base.lookup.as_str(), base.comparison.as_str()),
        ("miss", "absent")
    );
    fs::remove_dir_all(&baseline.dir).unwrap();

    let edited = new_scenario("G-edited");
    // Same-width literal edit: the references and declaration roster are unchanged.
    write_source(&edited, CONSUMER, &consumer_content(2));
    let result = worker(&edited, "s4");
    // The corrected key moves (bytes changed) so the answer recomputes ...
    assert_ne!(result.input_digest, base.input_digest);
    assert_eq!(
        (result.lookup.as_str(), result.comparison.as_str()),
        ("miss", "absent")
    );
    // ... but recomputes to an answer equal to the baseline.
    assert_eq!(
        result.output_digest, base.output_digest,
        "a reference-preserving body edit must recompute to an equal census answer"
    );

    fs::remove_dir_all(&edited.dir).unwrap();
}

// ---------------------------------------------------------------------------
// Scenario H control: a duplicate module is refused before the census answer is computed.
// ---------------------------------------------------------------------------

#[test]
fn duplicate_provider_refuses_before_the_census_answer() {
    let scenario = new_scenario("H");
    write_source(&scenario, CONSUMER, &consumer_content(1));
    // A duplicate module path — the WRONG provider: it is refused before S4 runs.
    write_source(&scenario, DUPLICATE, DUPLICATE_CONTENT);

    for derivation in ["s1", "s4"] {
        let result = worker(&scenario, derivation);
        assert!(result.refused, "{result:?}");
        assert_eq!(result.comparison, "duplicate-module");
        assert!(result.output_digest.is_empty());
        // No record is written for a refused compile.
        assert!(
            !record_path(&scenario, &result.input_digest).exists(),
            "a refused compile must not publish an answer"
        );
    }
    assert!(
        fs::read_dir(scenario.store.join("symbol-census-v1"))
            .unwrap()
            .next()
            .is_none(),
        "no census records for a refused duplicate-module tree"
    );

    fs::remove_dir_all(&scenario.dir).unwrap();
}
