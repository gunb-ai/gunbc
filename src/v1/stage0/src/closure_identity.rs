// Seed-retained keying (gunbc.design_argument::self_hosting seed-retained HAND-RUST). These derivations name WHAT a resolve or a
// witness is about -- a closure's content, the compiler that judges it, and a witness function
// over that closure -- and are consumed by the in-process resolved-graph memo, the typed-module
// key (std.interface_summary typed_module_key), the floor's per-row work key and the emitted
// compiler's identity. They are keying, not caching: they moved here unchanged from the deleted
// cross-process resolved-graph disk tier, whose store never served a hit after its JSON
// representation was disarmed. The seed strings below are preserved byte-for-byte so every key a
// receipt already carries stays the same key.
//
// Compiler identity is still the RUNNING BINARY's content hash. That is a known keying defect, not
// a property of this module: the binary embeds the checkout's commit, so a corpus-only commit
// re-keys every module. Its replacement is compiler identity derived from what the compiler IS
// (source closure + toolchain, the v2.compiler.self_host.generation axes) through the sealed
// per-axis observations; until that lands this function is the one authority every key consumes.

use std::fs;
use std::rc::Rc;
use std::sync::OnceLock;

use sha2::Digest;

use crate::v1_compiler_compile::SourceFile;
use crate::v1_rt::{self, Hash};

fn extract_module_path(content: &str) -> Option<String> {
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

pub fn closure_content_digest(sources: &[Rc<SourceFile>]) -> Hash {
    let mut pairs: Vec<(String, &str)> = sources
        .iter()
        .map(|s| {
            let module = extract_module_path(&s.content).unwrap_or_else(|| s.path.clone());
            (module, s.content.as_str())
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut acc = v1_rt::atom_identity_hash("resolved-graph-closure-v1".to_string());
    for (module, content) in pairs {
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(module));
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(content.to_string()));
    }
    acc
}

/// Content hash of the running compiler binary — the compiler-identity key term.
/// One authority for every key that must invalidate across a seed rebuild: the
/// resolved-graph subject digest (below) and the typed-module content key
/// (`std.interface_summary.typed_module_key`) both consume this digest rather than
/// re-deriving the executable read.
pub fn transform_content_digest() -> Hash {
    static DIGEST: OnceLock<Hash> = OnceLock::new();
    DIGEST
        .get_or_init(|| {
            // THE RUNNING IMAGE, NOT THE PATH IT WAS STARTED FROM. On Linux `current_exe()` is
            // the start path, which reads `<path> (deleted)` once a cargo build re-links it.
            // Measured 2026-08-30 (BuildBuddy, regen round cost): the in-round seed build
            // re-linked a byte-identical binary and this read panicked on the stale path.
            // `/proc/self/exe` resolves to the mapped image even after the directory entry is
            // replaced; elsewhere the start path is the only spelling available.
            let exe = if cfg!(target_os = "linux") {
                std::path::PathBuf::from("/proc/self/exe")
            } else {
                std::env::current_exe().unwrap_or_else(|e| {
                    panic!(
                        "closure identity: cannot locate compiler executable to content-address \
                         the transform: {e}"
                    )
                })
            };
            let bytes = fs::read(&exe).unwrap_or_else(|e| {
                panic!(
                    "closure identity: cannot read compiler executable {:?} to content-address \
                     the transform: {}",
                    exe, e
                )
            });
            v1_rt::bytes_identity_hash(&bytes)
        })
        .clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInputAxis {
    ClosureSubject,
    TransformContent,
}

impl KeyInputAxis {
    pub const ALL: &'static [KeyInputAxis] =
        &[KeyInputAxis::ClosureSubject, KeyInputAxis::TransformContent];
}

#[derive(Debug, Clone)]
pub struct KeyInputMaterials {
    closure_subject: Hash,
    transform_content: Hash,
}

impl KeyInputMaterials {
    pub fn new(closure_subject: Hash, transform_content: Hash) -> Self {
        Self {
            closure_subject,
            transform_content,
        }
    }

    fn materialize(&self, axis: KeyInputAxis) -> Hash {
        match axis {
            KeyInputAxis::ClosureSubject => self.closure_subject.clone(),
            KeyInputAxis::TransformContent => self.transform_content.clone(),
        }
    }
}

pub fn derive_subject_digest(materials: &KeyInputMaterials) -> Hash {
    KeyInputAxis::ALL.iter().fold(
        v1_rt::atom_identity_hash("resolved-graph-subject-v1".to_string()),
        |acc, axis| v1_rt::hash_combine(acc, materials.materialize(*axis)),
    )
}

pub fn subject_digest_for_closure(sources: &[Rc<SourceFile>]) -> Hash {
    let materials =
        KeyInputMaterials::new(closure_content_digest(sources), transform_content_digest());
    derive_subject_digest(&materials)
}

pub fn witness_work_subject_key(closure_subject_digest: &str, function: &str) -> Hash {
    v1_rt::hash_combine(
        v1_rt::atom_identity_hash(closure_subject_digest.to_string()),
        v1_rt::atom_identity_hash(function.to_string()),
    )
}

// --- census-reuse key (the v1 compiler reuse experiment) -------------------------------
//
// The existing `hash_combine` chain above is FNV-16-hex and is preserved byte-for-byte so the
// keys already carried by receipts stay stable. A reusable `build_symbol_index_census` answer,
// however, is addressed in `std.fabric_blob`, which requires a 64-hex lower-case SHA-256. This
// is intended to be the single authority for that key, so the fixture and the production
// serving path share one definition rather than each deriving a key. Today the only caller is
// the fixture (`materialization_shadow_cache_key`); the production serving path is a separate,
// not-yet-landed lane and does not call this yet. It is keying, not caching — it never supplies
// an answer to the compiler.

const CENSUS_KEY_SCHEMA: &[u8] = b"symbol-census-key-v1";
const CENSUS_QUESTION: &[u8] = b"v1.symbol.index::build_symbol_index_census";

fn census_key_field(hash: &mut sha2::Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn running_compiler_image_sha256() -> String {
    // THE RUNNING IMAGE, not the path it was started from — the same reason `transform_content_digest`
    // reads `/proc/self/exe`: a cargo relink replaces the directory entry behind `current_exe()`.
    let exe = if cfg!(target_os = "linux") {
        std::path::PathBuf::from("/proc/self/exe")
    } else {
        std::env::current_exe().unwrap_or_else(|e| {
            panic!("census key: cannot locate the running compiler to address its image: {e}")
        })
    };
    let bytes = fs::read(&exe)
        .unwrap_or_else(|e| panic!("census key: cannot read running compiler {exe:?}: {e}"));
    let mut hash = sha2::Sha256::new();
    hash.update(&bytes);
    hex::encode(hash.finalize())
}

/// THE cache key for a reusable `v1.symbol.index::build_symbol_index_census` answer.
///
/// Preimage, length-framed (each field is its byte length then its bytes), in this order:
///   schema tag, question tag, running compiler image SHA-256, the pool entry count, the closure
///   roster count, each closure module path (sorted), then each `(module path, source bytes)` of
///   the pool (sorted by module path).
///
/// The closure roster is load-bearing: the census is built over the ENTRY'S CLOSURE module set,
/// not the whole pool. Two different entries over the same pool but different closures produce a
/// different `SymbolIndex`, so a pool-only key would omit a declared input the answer depends on.
///
/// Guarantees required by the reuse contract:
///   * returns a 64-hex lower-case SHA-256;
///   * both the pool and the closure roster are sorted, so the same tree keyed in separate
///     processes is identical (never a `HashSet`/`HashMap` iteration order);
///   * it is sensitive to a newly added provider module (a new entry changes the set and key);
///   * the preimage contains no host file path, mtime or locator — only module paths and bytes.
///
/// Callers pass the process-shared module pool as `(module path, source bytes)` pairs plus the
/// sorted closure roster of authored module paths the census consumed. File paths, the per-file
/// newline index and the resolved graph are derived from these bytes and must NOT enter the
/// preimage.
pub fn symbol_census_key(modules: &[(String, String)], closure_roster: &[String]) -> String {
    let image = running_compiler_image_sha256();
    let mut sorted: Vec<(&String, &String)> = modules.iter().map(|(m, c)| (m, c)).collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let mut roster: Vec<&String> = closure_roster.iter().collect();
    roster.sort();
    let mut hash = sha2::Sha256::new();
    census_key_field(&mut hash, CENSUS_KEY_SCHEMA);
    census_key_field(&mut hash, CENSUS_QUESTION);
    census_key_field(&mut hash, image.as_bytes());
    census_key_field(&mut hash, &(sorted.len() as u64).to_le_bytes());
    census_key_field(&mut hash, &(roster.len() as u64).to_le_bytes());
    for module in roster {
        census_key_field(&mut hash, module.as_bytes());
    }
    for (module, content) in sorted {
        census_key_field(&mut hash, module.as_bytes());
        census_key_field(&mut hash, content.as_bytes());
    }
    hex::encode(hash.finalize())
}
