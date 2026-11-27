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
use std::io::{self, Read};
use std::path::Path;
use std::rc::Rc;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

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

const CENSUS_KEY_SCHEMA: &[u8] = b"symbol-census-key-v2";
const CENSUS_QUESTION: &[u8] = b"v1.symbol.index::build_symbol_index_census";

fn census_key_field(hash: &mut sha2::Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

/// THE cache key for a reusable `v1.symbol.index::build_symbol_index_census` answer.
///
/// Preimage, length-framed (each field is its byte length then its bytes), in this order:
///   schema tag, question tag, the compiler-image identity digest consumed from
///   `transform_content_digest` (the single authority — this does NOT re-read or re-hash the
///   executable), the pool entry count, the closure roster count, each closure module path
///   (sorted), then for each pool source, its `(source file path, module path, source bytes)`
///   triple (sorted by file path then module path).
///
/// Two inputs are load-bearing beyond content bytes:
///   * The closure roster — the census is built over the ENTRY'S CLOSURE module set, not the
///     whole pool; two different entries over the same pool but different closures produce a
///     different `SymbolIndex`.
///   * The source file path, bound JOINTLY with the module path and bytes. The `SymbolIndex`
///     carries source-located nodes and is consumed through the per-file `NewlineIndex` map, so a
///     rename of a source file (same bytes, same module declaration) changes the answer. Keying
///     only on module path + bytes would then serve a stale answer under an unchanged key (the
///     rename control in the fixture). Binding file path with the module and bytes — not as a
///     separate set — is what prevents a swapped file↔module↔bytes assignment from colliding.
///
/// Guarantees required by the reuse contract:
///   * returns a 64-hex lower-case SHA-256;
///   * the pool triples and the closure roster are sorted, so the same tree keyed in separate
///     processes is identical (never a `HashSet`/`HashMap` iteration order);
///   * it is sensitive to a newly added provider module and to a source-file rename;
///   * the preimage contains only repository-relative source paths, module paths and bytes — no
///     absolute host path, mtime or locator.
///
/// Callers pass each source the producer actually read as a `(source file path, module path,
/// source bytes)` triple, plus the sorted closure roster of authored module paths the census
/// consumed. The per-file newline index and the resolved graph are derived from these inputs and
/// must NOT enter the preimage.
pub fn symbol_census_key(
    modules: &[(String, String, String)],
    closure_roster: &[String],
) -> String {
    let image = transform_content_digest();
    let mut sorted: Vec<&(String, String, String)> = modules.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
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
    for (file_path, module_path, content) in sorted {
        census_key_field(&mut hash, file_path.as_bytes());
        census_key_field(&mut hash, module_path.as_bytes());
        census_key_field(&mut hash, content.as_bytes());
    }
    hex::encode(hash.finalize())
}

// --- one shadow-record format (the v1 compiler reuse experiments) ---------------------
//
// The production observer (`cli_run::materialization_shadow`) and the S1/S4 cache-key fixture
// both store fixed-size, integrity-checked digests of a compiler answer. This is the SINGLE
// authority for that on-disk format — record layout, SHA-256 integrity, create-only publish and
// the read/compare state machine — so the two experiments cannot drift apart (DESIGN
// §Single authority). It is parameterised only by a caller-supplied schema tag, so a question
// with a different schema shares the one format. It stores digests only; it never feeds an answer
// back to the compiler.

/// A 32-byte content digest used as a shadow record key or value.
pub type ShadowFingerprint = [u8; 32];

/// State of the record that may already exist for a key before a round runs.
pub enum ShadowPrior {
    Absent,
    Observed(ShadowFingerprint),
    Corrupt,
    Unreadable(io::Error),
}

/// Outcome of comparing the stored prior against the fresh answer.
pub enum ShadowObservation {
    Published,
    Equal,
    Different(&'static str),
    Corrupt,
    Io(&'static str, io::Error),
}

/// A 64-byte record: the 32-byte answer followed by a SHA-256 integrity tag over
/// `schema || key || value`.
pub fn shadow_record(
    schema: &[u8],
    key: &ShadowFingerprint,
    value: &ShadowFingerprint,
) -> [u8; 64] {
    let mut hash = Sha256::new();
    hash.update(schema);
    hash.update(key);
    hash.update(value);
    let mut bytes = [0; 64];
    bytes[..32].copy_from_slice(value);
    bytes[32..].copy_from_slice(&hash.finalize());
    bytes
}

/// Read the record at `path` for `key`, distinguishing absent / intact / corrupt / unreadable.
pub fn shadow_read_prior(path: &Path, schema: &[u8], key: &ShadowFingerprint) -> ShadowPrior {
    let mut open = fs::OpenOptions::new();
    open.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // A damaged experiment entry can be a FIFO or symlink. It must not block the compiler.
        open.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    let file = match open.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return ShadowPrior::Absent,
        Err(error) => return ShadowPrior::Unreadable(error),
    };
    match file.metadata() {
        Ok(metadata) if metadata.is_file() && metadata.len() == 64 => (),
        Ok(_) => return ShadowPrior::Corrupt,
        Err(error) => return ShadowPrior::Unreadable(error),
    }
    let mut bytes = Vec::new();
    if let Err(error) = file.take(65).read_to_end(&mut bytes) {
        return ShadowPrior::Unreadable(error);
    }
    let Ok(slice) = <&[u8; 64]>::try_from(bytes.as_slice()) else {
        return ShadowPrior::Corrupt;
    };
    let mut value = [0; 32];
    value.copy_from_slice(&slice[..32]);
    if &shadow_record(schema, key, &value) == slice {
        ShadowPrior::Observed(value)
    } else {
        ShadowPrior::Corrupt
    }
}

/// Compare the stored prior against the fresh answer and, when absent, publish a new record
/// atomically (create-only). A disagreement never overwrites its evidence.
pub fn shadow_compare_and_record(
    path: &Path,
    schema: &[u8],
    key: &ShadowFingerprint,
    prior: ShadowPrior,
    fresh: &ShadowFingerprint,
) -> ShadowObservation {
    match prior {
        ShadowPrior::Absent => {
            // Atomic create-only. A concurrent winner is read and checked against the
            // actual fresh result. Never overwrite a discrepancy or a damaged record.
            let publish = path
                .to_str()
                .ok_or_else(|| io::Error::other("shadow path is not UTF-8"))
                .and_then(|p| {
                    crate::v1_rt::gunbc_file_write_create_new(
                        p,
                        &shadow_record(schema, key, fresh),
                        None,
                    )
                });
            match publish {
                Ok(()) => ShadowObservation::Published,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    match shadow_read_prior(path, schema, key) {
                        ShadowPrior::Observed(value) if value == *fresh => {
                            ShadowObservation::Published
                        }
                        ShadowPrior::Observed(_) => ShadowObservation::Different("raced"),
                        ShadowPrior::Corrupt => ShadowObservation::Corrupt,
                        ShadowPrior::Unreadable(e) => ShadowObservation::Io("unreadable", e),
                        ShadowPrior::Absent => ShadowObservation::Io(
                            "unreadable",
                            io::Error::other("concurrent record disappeared"),
                        ),
                    }
                }
                Err(error) => ShadowObservation::Io("unwritable", error),
            }
        }
        ShadowPrior::Observed(value) if value == *fresh => ShadowObservation::Equal,
        ShadowPrior::Observed(_) => ShadowObservation::Different("hit"),
        ShadowPrior::Corrupt => ShadowObservation::Corrupt,
        ShadowPrior::Unreadable(error) => ShadowObservation::Io("unreadable", error),
    }
}
